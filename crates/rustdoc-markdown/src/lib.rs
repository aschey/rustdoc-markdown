use rustdoc_types::{
    Constant, Crate, DynTrait, Function, GenericArg, GenericArgs, GenericBound, GenericParamDef,
    GenericParamDefKind, Item, ItemEnum, Path, PolyTrait, StructKind, Term, TraitBoundModifier,
    Type, TypeBinding, TypeBindingKind, Visibility,
};

trait ToRepr {
    fn to_repr(&self) -> String;
}

pub fn build(path: &str) {
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .all_features(true)
        .manifest_path(path)
        .build()
        .unwrap();
    let json_string = std::fs::read_to_string(json_path).unwrap();
    let crate_docs: rustdoc_types::Crate = serde_json::from_str(&json_string).unwrap();

    for (id, item) in &crate_docs.index {
        if item.id == crate_docs.root {
            let res = process_item(&crate_docs, item);
            println!("{res:?}");
        }
    }
}

fn process_item(crate_docs: &Crate, item: &Item) -> Vec<String> {
    if item.visibility != Visibility::Public {
        return Vec::new();
    }
    match &item.inner {
        ItemEnum::Module(module) => {
            let mut res = Vec::new();
            for id in &module.items {
                let item = &crate_docs.index[id];
                res.push(process_item(crate_docs, item));
            }
            res.into_iter().flatten().collect()
        }
        ItemEnum::ExternCrate { name, rename } => todo!(),
        ItemEnum::Import(_) => Vec::new(),
        ItemEnum::Union(_) => Vec::new(),
        ItemEnum::Struct(struct_) => {
            let name = item.name.clone().unwrap();
            let non_exhaustive = item
                .attrs
                .iter()
                .find(|a| *a == "#[non_exhaustive]")
                .map(|s| s.to_string() + "\n")
                .unwrap_or_default();
            let struct_repr = match &struct_.kind {
                StructKind::Unit => format!("pub struct {name}"),
                StructKind::Tuple(ids) => {
                    let tuple_fields: Vec<_> = ids
                        .iter()
                        .filter_map(|id| {
                            id.as_ref().map(|id| {
                                let item = &crate_docs.index[id];
                                let processed = process_item(crate_docs, item);
                                processed.first().unwrap().to_string()
                            })
                        })
                        .collect();
                    format!("pub struct {name}({})", comma_separated(&tuple_fields))
                }
                StructKind::Plain {
                    fields,
                    fields_stripped,
                } => {
                    let mut s = format!("{non_exhaustive}pub struct {name} {{\n");
                    let fields_processed = fields.iter().map(|id| {
                        let item = &crate_docs.index[id];
                        let processed = process_item(crate_docs, item);
                        let item = processed.first().unwrap().to_string();
                        item
                    });
                    for field in fields_processed {
                        s += &format!("    {field},\n");
                    }
                    s += "}";
                    s
                }
            };
            vec![struct_repr]
        }
        ItemEnum::StructField(ty) => {
            let name = item.name.clone().unwrap();
            let s = format!("pub {name}: {}", ty.to_repr());
            vec![s]
        }
        ItemEnum::Enum(_) => todo!(),
        ItemEnum::Variant(_) => todo!(),
        ItemEnum::Function(func) => {
            let name = item.name.clone().unwrap();
            let inputs: Vec<_> = func
                .decl
                .inputs
                .iter()
                .map(|(name, ty)| format!("{name}: {}", ty.to_repr()))
                .collect();
            let inputs = comma_separated(&inputs);
            let async_ = if func.header.async_ { "async " } else { "" };
            let output = func
                .decl
                .output
                .as_ref()
                .map(|o| format!(" -> {}", o.to_repr()))
                .unwrap_or_default();
            vec![format!("pub {async_}fn {name}({inputs}){output}")]
        }
        ItemEnum::Trait(_) => todo!(),
        ItemEnum::TraitAlias(_) => todo!(),
        ItemEnum::Impl(_) => todo!(),
        ItemEnum::TypeAlias(_) => todo!(),
        ItemEnum::OpaqueTy(_) => todo!(),
        ItemEnum::Constant { type_, const_ } => todo!(),
        ItemEnum::Static(_) => todo!(),
        ItemEnum::ForeignType => todo!(),
        ItemEnum::Macro(_) => todo!(),
        ItemEnum::ProcMacro(_) => todo!(),
        ItemEnum::Primitive(_) => todo!(),
        ItemEnum::AssocConst { type_, default } => todo!(),
        ItemEnum::AssocType {
            generics,
            bounds,
            default,
        } => todo!(),
    }
}

impl ToRepr for Type {
    fn to_repr(&self) -> String {
        match self {
            Type::ResolvedPath(path) => path.to_repr(),
            Type::DynTrait(dyn_trait) => dyn_trait.to_repr(),
            Type::Generic(generic) => generic.to_string(),
            Type::Primitive(val) => val.clone(),
            Type::FunctionPointer(_) => todo!(),
            Type::Tuple(_) => todo!(),
            Type::Slice(slice) => format!("[{}]", slice.to_repr()),
            Type::Array { type_, len } => {
                format!("[{}; {len}]", type_.to_repr())
            }
            Type::Pat {
                type_,
                __pat_unstable_do_not_use,
            } => todo!(),
            Type::ImplTrait(bounds) => {
                let bounds: Vec<_> = bounds.iter().map(|b| b.to_repr()).collect();

                format!("impl {}", plus_separated(&bounds))
            }
            Type::Infer => todo!(),
            Type::RawPointer { mutable, type_ } => {
                let mutability = if *mutable { "*mut" } else { "*const" };
                format!("{mutability} {}", type_.to_repr())
            }
            Type::BorrowedRef {
                lifetime,
                mutable,
                type_,
            } => {
                let lifetime = lifetime
                    .as_ref()
                    .map(|l| format!("'{l} "))
                    .unwrap_or_default();
                let mutable = if *mutable {
                    "mut ".to_string()
                } else {
                    "".to_string()
                };
                format!("{lifetime}{mutable}{}", type_.to_repr())
            }
            Type::QualifiedPath {
                name,
                args,
                self_type,
                trait_,
            } => todo!(),
        }
    }
}

impl ToRepr for DynTrait {
    fn to_repr(&self) -> String {
        let mut s = "dyn ".to_string();
        s += &comma_separated(&self.traits);
        if let Some(lifetime) = &self.lifetime {
            s += &format!(" {lifetime}");
        }
        s
    }
}

impl ToRepr for PolyTrait {
    fn to_repr(&self) -> String {
        let mut s = self.trait_.to_repr();
        let generics = comma_separated(&self.generic_params);
        if !generics.is_empty() {
            s = format!("{generics} {s}");
        }
        s
    }
}

impl ToRepr for GenericParamDef {
    fn to_repr(&self) -> String {
        match &self.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                format!("'{}: {}", self.name, plus_separated(outlives))
            }
            GenericParamDefKind::Type {
                bounds,
                default,
                synthetic,
            } => {
                if *synthetic {
                    "".to_owned()
                } else {
                    let mut s = comma_separated(bounds);
                    if let Some(default) = default {
                        s += &format!(" = {}", default.to_repr());
                    }
                    s
                }
            }
            GenericParamDefKind::Const { type_, default } => {
                let mut s = format!("const {}", type_.to_repr());
                if let Some(default) = default {
                    s += &format!(" = {default}");
                }
                s
            }
        }
    }
}

// impl ToRepr for GenericParamDefKind {
//     fn to_repr(&self) -> String {
//         match self {
//             GenericParamDefKind::Lifetime { outlives } => format!("'"),
//             GenericParamDefKind::Type {
//                 bounds,
//                 default,
//                 synthetic,
//             } => todo!(),
//             GenericParamDefKind::Const { type_, default } => todo!(),
//         }
//     }
// }

impl ToRepr for GenericBound {
    fn to_repr(&self) -> String {
        match self {
            GenericBound::TraitBound {
                trait_,
                generic_params,
                modifier,
            } => {
                let trait_ = trait_.to_repr();
                let generic_params = if !generic_params.is_empty() {
                    format!(
                        "<{}>",
                        comma_separated(
                            &generic_params
                                .iter()
                                .map(|p| p.to_repr())
                                .collect::<Vec<_>>()
                        )
                    )
                } else {
                    "".to_string()
                };
                format!("{trait_}{generic_params}{}", modifier.to_repr())
            }
            GenericBound::Outlives(lifetime) => format!("'{lifetime}"),
            GenericBound::Use(use_) => {
                let use_ = comma_separated(use_);
                format!("use<{use_}>")
            }
        }
    }
}

impl ToRepr for TraitBoundModifier {
    fn to_repr(&self) -> String {
        match self {
            TraitBoundModifier::None => "".to_string(),
            TraitBoundModifier::Maybe => "?".to_string(),
            TraitBoundModifier::MaybeConst => "?const".to_string(),
        }
    }
}

impl ToRepr for String {
    fn to_repr(&self) -> String {
        self.clone()
    }
}

impl<T> ToRepr for Box<T>
where
    T: ToRepr,
{
    fn to_repr(&self) -> String {
        (**self).to_repr()
    }
}

impl ToRepr for Path {
    fn to_repr(&self) -> String {
        let mut s = self.name.to_string();
        s += &self.args.to_repr();
        s
    }
}

impl ToRepr for GenericArgs {
    fn to_repr(&self) -> String {
        match self {
            GenericArgs::AngleBracketed { args, bindings } => {
                let mut s = "".to_string();
                if !args.is_empty() {
                    s += &comma_separated(args);
                }
                if !bindings.is_empty() {
                    if !s.is_empty() {
                        s += ", ";
                    }
                    s += &comma_separated(bindings);
                }
                if !s.is_empty() {
                    format!("<{s}>")
                } else {
                    "".to_string()
                }
            }
            GenericArgs::Parenthesized { inputs, output } => {
                let mut s = format!("({})", comma_separated(inputs));
                if let Some(output) = output {
                    s += &format!(" -> {}", output.to_repr());
                }
                s
            }
        }
    }
}

impl ToRepr for GenericArg {
    fn to_repr(&self) -> String {
        match self {
            GenericArg::Lifetime(lifetime) => lifetime.to_owned(),
            GenericArg::Type(ty) => ty.to_repr(),
            GenericArg::Const(constant) => constant.to_repr(),
            GenericArg::Infer => "_".to_string(),
        }
    }
}

fn comma_separated<T>(t: &[T]) -> String
where
    T: ToRepr,
{
    t.iter().map(|g| g.to_repr()).collect::<Vec<_>>().join(", ")
}

fn plus_separated<T>(t: &[T]) -> String
where
    T: ToRepr,
{
    t.iter()
        .map(|g| g.to_repr())
        .collect::<Vec<_>>()
        .join(" + ")
}

impl ToRepr for Constant {
    fn to_repr(&self) -> String {
        todo!()
    }
}

impl ToRepr for TypeBinding {
    fn to_repr(&self) -> String {
        let mut s = self.name.clone();
        let args = self.args.to_repr();
        let binding = self.binding.to_repr();
        format!("{s} {args} {binding}")
    }
}

impl ToRepr for TypeBindingKind {
    fn to_repr(&self) -> String {
        match self {
            TypeBindingKind::Equality(term) => term.to_repr(),
            TypeBindingKind::Constraint(bound) => comma_separated(bound),
        }
    }
}

impl ToRepr for Term {
    fn to_repr(&self) -> String {
        match self {
            Term::Type(ty) => ty.to_repr(),
            Term::Constant(constant) => constant.to_repr(),
        }
    }
}

impl<T> ToRepr for Option<T>
where
    T: ToRepr,
{
    fn to_repr(&self) -> String {
        match self {
            Some(t) => t.to_repr(),
            None => "".to_string(),
        }
    }
}
