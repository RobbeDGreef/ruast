use insta::assert_snapshot;
use ruast::*;

#[test]
fn test() {
    let mut krate = Crate::new();
    let def = Fn::main(
        None,
        Block::from(
            Path::single("println")
                .mac_call(vec![Token::lit("Hello, world!")])
                .semi(),
        ),
    );
    krate.add_item(def);
    assert_snapshot!(krate, @r###"
    fn main() {
        println!("Hello, world!");
    }
    "###);
    krate.try_remove_item_by_id("main");
    assert!(krate.is_empty());
    assert_snapshot!(krate, @"");
}

#[test]
fn test_general() {
    let mut krate = Crate::new();
    let i = krate.add_item(Fn {
        is_unsafe: false,
        is_const: false,
        is_async: false,
        abi: None,
        ident: "main".to_string(),
        generics: vec![],
        fn_decl: FnDecl::regular(vec![], None),
        body: Some(Block::from(Stmt::Semi(Semi::new(Expr::new(MacCall {
            path: Path::single("println"),
            args: DelimArgs::from(vec![Token::lit("Hello, world!")]),
        }))))),
    });
    assert_snapshot!(krate, @r###"
    fn main() {
        println!("Hello, world!");
    }
    "###);
    assert_snapshot!(krate[i], @r###"
    fn main() {
        println!("Hello, world!");
    }
    "###);
}

#[test]
fn test_blocks() {
    let block = Block::from(Stmt::Expr(Expr::new(Lit::int("17"))));
    assert_snapshot!(block, @"
    {
        17
    }");

    assert_snapshot!(Block::empty(), @"{}");
}

#[test]
fn test_if_else() {
    let if_else = If::new(
        Expr::new(Lit::bool("true")),
        Block::from(Stmt::Expr(Expr::new(Lit::int("17")))),
        Some(Expr::from(Block::from(Stmt::Expr(Expr::new(Lit::int(
            "39",
        )))))),
    );
    assert_snapshot!(if_else, @"
    if true {
        17
    } else {
        39
    }");
    let if_elseif_else = If::new(
        Expr::new(Lit::bool("false")),
        Block::from(Stmt::Expr(Expr::new(Lit::int("1")))),
        Some(Expr::from(if_else)),
    );
    assert_snapshot!(if_elseif_else, @"
    if false {
        1
    } else if true {
        17
    } else {
        39
    }");
}

#[test]
fn test_binop() {
    let lhs = Lit::int("1");
    let rhs = Lit::int("2");
    let add = lhs.clone().add(rhs.clone());
    assert_snapshot!(add, @"1 + 2");

    let add_add = add.clone().add(Lit::int("3"));
    assert_snapshot!(add_add, @"1 + 2 + 3");

    let mul_add = add.mul(Lit::int("3"));
    assert_snapshot!(mul_add, @"(1 + 2) * 3");

    let mul = lhs.clone().mul(rhs.clone());
    let add_mul = mul.add(Lit::int("3"));
    assert_snapshot!(add_mul, @"1 * 2 + 3");

    let add = lhs.neg().add(rhs.neg());
    assert_snapshot!(add, @"-1 + -2");
}

#[test]
fn test_try() {
    let x = Path::single("x");
    let try_ = x.clone().try_();
    assert_snapshot!(try_, @"x?");

    let try_add = try_.clone().add(Lit::int("42"));
    assert_snapshot!(try_add, @"x? + 42");

    let neg_try = x.neg().try_();
    assert_snapshot!(neg_try, @"(-x)?");

    let try_neg = try_.clone().neg();
    assert_snapshot!(try_neg, @"-x?");

    let try_call = try_.call(vec![Lit::int("42").into()]);
    assert_snapshot!(try_call, @"(x?)(42)");
}

#[test]
fn test_addrof() {
    let x = Path::single("x");
    let add = x.clone().add(Path::single("y"));
    let ref_ = add.clone().ref_immut();
    assert_snapshot!(ref_, @"&(x + y)");
    let ref_mut = add.ref_mut();
    assert_snapshot!(ref_mut, @"&mut (x + y)");

    let addr_deref = x.ref_immut().deref();
    assert_snapshot!(addr_deref, @"*&x");
}

#[test]
fn test_field() {
    let x = Path::single("x");
    let add = x.clone().add(Path::single("y"));
    let field = add.field("z");
    assert_snapshot!(field, @"(x + y).z");
    let ref_ = x.clone().ref_immut();
    let ref_field = ref_.field("z");
    assert_snapshot!(ref_field, @"(&x).z");
    let field_ref = x.clone().field("z").ref_immut();
    assert_snapshot!(field_ref, @"&x.z");

    let field_raw_ref = x
        .clone()
        .field("z")
        .addr_of(BorrowKind::Raw, Mutability::Mut);
    assert_snapshot!(field_raw_ref, @"&raw mut x.z");
    let raw_ref_field = x.addr_of(BorrowKind::Raw, Mutability::Mut).field("z");
    assert_snapshot!(raw_ref_field, @"(&raw mut x).z");
}

#[test]
fn test_closure() {
    let x = Path::single("x");
    let decl = FnDecl::regular(vec![Param::ident("a", Type::Infer)], None);
    let closure = Closure::simple(decl, x);
    assert_snapshot!(closure, @"|a: _| { x }");

    let call = closure.call(vec![Lit::int("42").into()]);
    assert_snapshot!(call, @"(|a: _| { x })(42)");
}

#[test]
fn test_return() {
    let x = Path::single("x");
    let return_ = x.return_();
    assert_snapshot!(return_, @"return x");

    let call_return = return_.call(vec![]);
    assert_snapshot!(call_return, @"(return x)()");
}

#[test]
fn test_match() {
    let x = Path::single("x");
    let arm1 = Arm::new(Pat::Lit(Lit::int("1").into()), None, Lit::int("1"));
    let arm2 = Arm::new(Pat::Lit(Lit::int("2").into()), None, Lit::int("2"));
    let default_arm = Arm::new(Pat::Wild, None, Lit::int("0"));
    let match_ = Match::new(x, vec![arm1, arm2, default_arm]);
    assert_snapshot!(match_, @"match x {
    1 => 1,
    2 => 2,
    _ => 0,
}");
}

#[test]
fn test_use() {
    let path = Path::single("foo").chain("bar").chain("baz");
    let use_ = Use::from(path.clone());

    assert_snapshot!(use_, @"use foo::bar::baz;");

    let use_tree = UseTree::path(UsePath::from(path));
    let use_ = Use::from(use_tree);
    assert_snapshot!(use_, @"use foo::bar::baz;");

    let baz = UseTree::name("baz");
    let qux = UseTree::name("qux");
    let group = vec![baz, qux];
    let items = UseTree::Group(group.clone());
    let items = UseTree::Path(UsePath::new("bar", items));
    let tree = UseTree::Path(UsePath::new("foo", items));
    let use_ = Use::tree(tree);
    assert_snapshot!(use_, @"use foo::bar::{baz, qux};");

    let tree = Path::single("foo").chain("bar").chain_use_group(group);
    let use_ = Use::from(tree);
    assert_snapshot!(use_, @"use foo::bar::{baz, qux};");

    let tree = Path::single("foo").chain("bar").chain_use_glob();
    let use_ = Use::from(tree);
    assert_snapshot!(use_, @"use foo::bar::*;");
}

#[test]
fn test_visibility_scope() {
    let vis_crate = Visibility::crate_();
    assert_snapshot!(vis_crate, @"pub(crate) ");

    let vis_super = Visibility::super_();
    assert_snapshot!(vis_super, @"pub(super) ");

    let vis_self = Visibility::self_();
    assert_snapshot!(vis_self, @"pub(self) ");

    let path = Path::single("my_module");
    let vis_path = Visibility::in_path(path);
    assert_snapshot!(vis_path, @"pub(in my_module) ");

    let nested_path = Path::single("crate").chain("module").chain("submodule");
    let vis_nested_path = Visibility::in_path(nested_path);
    assert_snapshot!(vis_nested_path, @"pub(in crate::module::submodule) ");

    let vis_inherited = Visibility::default();
    assert_snapshot!(vis_inherited, @"");

    let vis_public = Visibility::Public;
    assert_snapshot!(vis_public, @"pub ");
}

#[test]
fn test_macro_call() {
    let tokens = vec![
        Token::Keyword(KeywordToken::Let),
        Token::Keyword(KeywordToken::Mut),
        Token::Ident("x".into()),
        Token::Eq,
        Token::Lit(Lit::int("42")),
    ];
    let mac_call =
        Path::single("assign").mac_call(DelimArgs::new(MacDelimiter::Parenthesis, tokens.into()));

    assert_snapshot!(mac_call, @"assign!(let mut x = 42)");
}

#[test]
fn test_joint_token() {
    let ts = TokenStream::from(vec![
        Token::Ident("foo".into()),
        Token::Dot,
        Token::Ident("bar".into()),
    ]);
    assert_snapshot!(ts, @"foo . bar");

    let ts = TokenStream::from(vec![
        Token::Ident("foo".into()),
        Token::Dot.into_joint(),
        Token::Ident("bar".into()),
    ]);
    assert_snapshot!(ts, @"foo .bar");

    let ts = TokenStream::from(vec![
        Token::Ident("foo".into()).into_joint(),
        Token::Dot.into_joint(),
        Token::Ident("bar".into()),
    ]);
    assert_snapshot!(ts, @"foo.bar");
}

#[test]
fn test_mutty_to_tokenstream() {
    let immut_ty = MutTy::immut(Type::i32());
    let ts = TokenStream::from(immut_ty);
    assert_snapshot!(ts, @"i32");

    let mut_ty = MutTy::mut_(Type::i32());
    let ts = TokenStream::from(mut_ty);
    assert_snapshot!(ts, @"mut i32");
}

#[test]
fn test_ref_to_tokenstream() {
    let ref_ty = Ref::new(None::<String>, MutTy::immut(Type::i32()));
    let ts = TokenStream::from(ref_ty);
    assert_snapshot!(ts, @"&i32");

    let ref_mut_ty = Ref::new(None::<String>, MutTy::mut_(Type::i32()));
    let ts = TokenStream::from(ref_mut_ty);
    assert_snapshot!(ts, @"&mut i32");

    let ref_lifetime_ty = Ref::new(Some("static"), MutTy::immut(Type::i32()));
    let ts = TokenStream::from(ref_lifetime_ty);
    assert_snapshot!(ts, @"&'static i32");
}

#[test]
fn test_ptrkind_to_tokenstream() {
    let const_ptr = PtrKind::Const;
    let ts = TokenStream::from(const_ptr);
    assert_snapshot!(ts, @"const");

    let mut_ptr = PtrKind::Mut;
    let ts = TokenStream::from(mut_ptr);
    assert_snapshot!(ts, @"mut");
}

#[test]
fn test_ptr_to_tokenstream() {
    let const_ptr = Ptr::new(PtrKind::Const, Type::i32());
    let ts = TokenStream::from(const_ptr);
    assert_snapshot!(ts, @"*const i32");

    let mut_ptr = Ptr::new(PtrKind::Mut, Type::i32());
    let ts = TokenStream::from(mut_ptr);
    assert_snapshot!(ts, @"*mut i32");
}

#[test]
fn test_barefn_to_tokenstream() {
    let simple_fn = BareFn::safe(vec![], vec![], Type::unit());
    let ts = TokenStream::from(simple_fn);
    assert_snapshot!(ts, @"fn () -> ()");

    let unsafe_fn = BareFn::new(vec![], vec![], Type::i32(), None, true);
    let ts = TokenStream::from(unsafe_fn);
    assert_snapshot!(ts, @"unsafe fn () -> i32");

    let extern_fn = BareFn::new(vec![], vec![], Type::i32(), Some("C".to_string()), false);
    let ts = TokenStream::from(extern_fn);
    assert_snapshot!(ts, @"extern \"C\" fn () -> i32");
}

#[test]
fn test_typeparam_to_tokenstream() {
    let simple_param = TypeParam::simple("T");
    let ts = TokenStream::from(simple_param);
    assert_snapshot!(ts, @"T");

    let bounded_param = TypeParam::new(
        "T",
        vec![GenericBound::Trait(PolyTraitRef::simple(Path::single(
            "Clone",
        )))],
    );
    let ts = TokenStream::from(bounded_param);
    assert_snapshot!(ts, @"T: Clone");
}

#[test]
fn test_constparam_to_tokenstream() {
    let const_param = ConstParam::new("N", Type::usize());
    let ts = TokenStream::from(const_param);
    assert_snapshot!(ts, @"const N: usize");
}

#[test]
fn test_polytraitref_to_tokenstream() {
    let simple_trait_ref = PolyTraitRef::simple(Path::single("Clone"));
    let ts = TokenStream::from(simple_trait_ref);
    assert_snapshot!(ts, @"Clone");

    let complex_trait_ref = PolyTraitRef::simple(Path::single("Iterator").chain("Item"));
    let ts = TokenStream::from(complex_trait_ref);
    assert_snapshot!(ts, @"Iterator::Item");
}

#[test]
fn test_genericbound_to_tokenstream() {
    let trait_bound = GenericBound::Trait(PolyTraitRef::simple(Path::single("Clone")));
    let ts = TokenStream::from(trait_bound);
    assert_snapshot!(ts, @"Clone");

    let lifetime_bound = GenericBound::Outlives("static".to_string());
    let ts = TokenStream::from(lifetime_bound);
    assert_snapshot!(ts, @"'static");
}

#[test]
fn test_traitobject_to_tokenstream() {
    let static_trait_obj = TraitObject::static_(vec![GenericBound::Trait(PolyTraitRef::simple(
        Path::single("Clone"),
    ))]);
    let ts = TokenStream::from(static_trait_obj);
    assert_snapshot!(ts, @"Clone");

    let dyn_trait_obj = TraitObject::dyn_(vec![GenericBound::Trait(PolyTraitRef::simple(
        Path::single("Clone"),
    ))]);
    let ts = TokenStream::from(dyn_trait_obj);
    assert_snapshot!(ts, @"dyn Clone");
}

#[test]
fn test_impltrait_to_tokenstream() {
    let impl_trait = ImplTrait::new(vec![GenericBound::Trait(PolyTraitRef::simple(
        Path::single("Clone"),
    ))]);
    let ts = TokenStream::from(impl_trait);
    assert_snapshot!(ts, @"impl Clone");
}

#[test]
fn test_type_to_tokenstream() {
    let slice_ty = Type::Slice(Box::new(Type::i32()));
    let ts = TokenStream::from(slice_ty);
    assert_snapshot!(ts, @"[i32]");

    let array_ty = Type::Array(
        Box::new(Type::i32()),
        Box::new(Const(Expr::new(Lit::int("5")))),
    );
    let ts = TokenStream::from(array_ty);
    assert_snapshot!(ts, @"[i32; 5]");

    let never_ty = Type::Never;
    let ts = TokenStream::from(never_ty);
    assert_snapshot!(ts, @"!");

    let tuple_ty = Type::Tuple(vec![Type::i32(), Type::bool()]);
    let ts = TokenStream::from(tuple_ty);
    assert_snapshot!(ts, @"(i32, bool)");

    let infer_ty = Type::Infer;
    let ts = TokenStream::from(infer_ty);
    assert_snapshot!(ts, @"_");
}
