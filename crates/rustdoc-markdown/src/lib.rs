use rustdoc_types::{
    Constant, Crate, DynTrait, GenericArg, GenericArgs, GenericBound, GenericParamDef,
    GenericParamDefKind, Generics, Item, ItemEnum, Path, PolyTrait, StructKind, Term,
    TraitBoundModifier, Type, TypeBinding, TypeBindingKind, Visibility, WherePredicate,
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
        if *id == crate_docs.root {
            let res = process_item(&crate_docs, item, false);
            println!("{res:#?}");
        }
    }
}

fn process_item(crate_docs: &Crate, item: &Item, allow_non_public: bool) -> Vec<String> {
    if !(item.visibility == Visibility::Public
        || (item.visibility == Visibility::Default && allow_non_public))
    {
        return Vec::new();
    }
    match &item.inner {
        ItemEnum::Module(module) => {
            let mut res = Vec::new();
            for id in &module.items {
                let item = &crate_docs.index[id];
                res.push(process_item(crate_docs, item, allow_non_public));
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
            let (generics, where_clause) = generics_repr(&struct_.generics);
            let mut vis = item.visibility.to_repr();
            if !vis.is_empty() {
                vis += " ";
            }
            let struct_repr = match &struct_.kind {
                StructKind::Unit => format!("{vis}struct {name}{generics}{where_clause}"),
                StructKind::Tuple(ids) => {
                    let tuple_fields: Vec<_> = ids
                        .iter()
                        .map(|id| {
                            if let Some(id) = id {
                                let mut item = crate_docs.index[id].clone();
                                // We don't want to show the numeric names for tuples
                                item.name = None;
                                let processed = process_item(crate_docs, &item, false);
                                processed.first().unwrap().to_string()
                            } else {
                                "_".to_string()
                            }
                        })
                        .collect();
                    format!(
                        "{vis}struct {name}{generics}({})",
                        comma_separated(&tuple_fields)
                    )
                }
                StructKind::Plain {
                    fields,
                    fields_stripped,
                } => {
                    let mut s =
                        format!("{non_exhaustive}pub struct {name}{generics}{where_clause} {{");
                    let fields_processed: Vec<_> = fields
                        .iter()
                        .map(|id| {
                            let item = &crate_docs.index[id];
                            let processed = process_item(crate_docs, item, false);
                            let item = processed.first().unwrap().to_string();
                            item
                        })
                        .collect();

                    for field in &fields_processed {
                        s += &format!("\n    {field},");
                    }
                    if *fields_stripped {
                        s += "/* private fields */";
                    }
                    if !fields_processed.is_empty() {
                        s += "\n";
                    }
                    s += "}";
                    s
                }
            };
            vec![struct_repr]
        }
        ItemEnum::StructField(ty) => {
            let mut vis = item.visibility.to_repr();
            if !vis.is_empty() {
                vis += " ";
            }
            let s = if let Some(name) = &item.name {
                format!("{vis}{name}: {}", ty.to_repr())
            } else {
                format!("{vis}{}", ty.to_repr())
            };
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
                .map(|(name, ty)| {
                    let ty = ty.to_repr();
                    if name == "self" && ty == "&Self" {
                        "&self".to_string()
                    } else if name == "self" && ty == "Self" {
                        "self".to_string()
                    } else {
                        format!("{name}: {ty}")
                    }
                })
                .collect();
            let inputs = comma_separated(&inputs);
            let const_ = if func.header.const_ { "const " } else { "" };
            let async_ = if func.header.async_ { "async " } else { "" };
            let unsafe_ = if func.header.unsafe_ { "unsafe " } else { "" };
            let (generics, where_clause) = generics_repr(&func.generics);
            let output = func
                .decl
                .output
                .as_ref()
                .map(|o| format!(" -> {}", o.to_repr()))
                .unwrap_or_default();
            let mut vis = item.visibility.to_repr();
            if !vis.is_empty() {
                vis += " ";
            }
            vec![format!(
                "{vis}{const_}{unsafe_}{async_}fn {name}{generics}({inputs}){output}{where_clause}"
            )]
        }
        ItemEnum::Trait(trait_) => {
            let name = item.name.clone().unwrap();
            let (generics, where_clause) = generics_repr(&trait_.generics);
            let auto = if trait_.is_auto { "auto " } else { "" };
            let unsafe_ = if trait_.is_unsafe { "unsafe " } else { "" };
            let mut vis = item.visibility.to_repr();
            if !vis.is_empty() {
                vis += " ";
            }
            let mut bounds = plus_separated(&trait_.bounds);
            if !bounds.is_empty() {
                bounds = format!(": {bounds}");
            }
            let mut s =
                format!("{vis}{auto}{unsafe_}trait {name}{generics}{bounds}{where_clause} {{");

            let items: Vec<_> = trait_
                .items
                .iter()
                .filter_map(|id| {
                    let item = &crate_docs.index[id];
                    let processed = process_item(crate_docs, item, true);
                    let item = processed.first().cloned();
                    item
                })
                .collect();
            for item in &items {
                s += &format!("\n    {item}");
            }
            s += "\n}";
            vec![s]
        }
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
        ItemEnum::AssocConst { type_, default } => {
            let name = item.name.clone().unwrap();
            let mut s = format!("const {name}: {}", type_.to_repr());
            if let Some(default) = default {
                s += &format!(" = {default}");
            }
            s += ";";

            vec![s]
        }
        ItemEnum::AssocType {
            generics,
            bounds,
            default,
        } => {
            let name = item.name.clone().unwrap();

            let (generics, where_clause) = generics_repr(generics);
            let mut s = format!("type {name}{generics}");
            if !bounds.is_empty() {
                s += &format!(" {}", plus_separated(bounds));
            }
            if let Some(default) = default {
                s += &format!(" = {}", default.to_repr());
            }
            s += &format!("{where_clause};");

            vec![s]
        }
    }
}

impl ToRepr for Visibility {
    fn to_repr(&self) -> String {
        match self {
            Visibility::Public => "pub".to_string(),
            Visibility::Default => "".to_string(),
            Visibility::Crate => "pub(crate)".to_string(),
            Visibility::Restricted { parent, path } => format!("pub(in {path})"),
        }
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
                    .map(|l| format!("{l} "))
                    .unwrap_or_default();
                let mutable = if *mutable {
                    "mut ".to_string()
                } else {
                    "".to_string()
                };
                format!("&{lifetime}{mutable}{}", type_.to_repr())
            }
            Type::QualifiedPath {
                name,
                args,
                self_type,
                trait_,
            } => {
                let mut s = self_type.to_repr();
                if let Some(trait_) = trait_ {
                    let trait_ = trait_.to_repr();
                    if !trait_.is_empty() {
                        s = format!("<{s} as {trait_}>");
                    }
                }
                let args = args.to_repr();
                format!("{s}::{name}{args}")
            }
        }
    }
}

impl ToRepr for DynTrait {
    fn to_repr(&self) -> String {
        let mut s = plus_separated(&self.traits);
        let mut num_items = self.traits.len();
        if let Some(lifetime) = &self.lifetime {
            num_items += 1;
            s += &format!(" + {lifetime}");
        }
        // dyn trait args need parenthesis if there's > 1 value
        if num_items > 1 {
            s = format!("({s})");
        }
        format!("dyn {s}")
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
        let kind = match &self.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                // let mut s = self.name.to_string();
                if outlives.is_empty() {
                    "".to_string()
                } else {
                    // format!(": {}", plus_separated(outlives))
                    plus_separated(outlives)
                }
            }
            GenericParamDefKind::Type {
                bounds,
                default,
                synthetic,
            } => {
                if *synthetic {
                    "".to_owned()
                } else {
                    let mut s = plus_separated(bounds);
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
        };
        let mut s = self.name.clone();
        if !kind.is_empty() {
            s += &format!(": {kind}");
        }
        s
    }
}

impl ToRepr for WherePredicate {
    fn to_repr(&self) -> String {
        match self {
            WherePredicate::BoundPredicate {
                type_,
                bounds,
                generic_params,
            } => {
                let mut s = "".to_string();
                let type_ = type_.to_repr();
                let bounds = plus_separated(bounds);
                if !generic_params.is_empty() {
                    let generic_params = comma_separated(generic_params);
                    s += &format!("for <{generic_params}>");
                }
                format!("{s}{type_}: {bounds}")
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                let outlives = plus_separated(outlives);
                format!("{lifetime} {outlives}")
            }
            WherePredicate::EqPredicate { lhs, rhs } => {
                format!("{} = {}", lhs.to_repr(), rhs.to_repr())
            }
        }
    }
}

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

fn generics_repr(generics: &Generics) -> (String, String) {
    let generic_params = if generics.params.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", comma_separated(&generics.params))
    };
    let where_clause = if generics.where_predicates.is_empty() {
        "".to_string()
    } else {
        format!(" where {}", comma_separated(&generics.where_predicates))
    };
    (generic_params, where_clause)
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
        if !args.is_empty() {
            s += &format!(" {args}");
        }
        let binding = self.binding.to_repr();
        if !binding.is_empty() {
            s += &format!(" {binding}");
        }
        s
    }
}

impl ToRepr for TypeBindingKind {
    fn to_repr(&self) -> String {
        match self {
            TypeBindingKind::Equality(term) => format!("= {}", term.to_repr()),
            TypeBindingKind::Constraint(bounds) => plus_separated(bounds),
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
