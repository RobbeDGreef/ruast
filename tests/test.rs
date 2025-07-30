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
    assert_snapshot!(ts, @"fn() -> ()");

    let unsafe_fn = BareFn::new(vec![], vec![], Type::i32(), None, true);
    let ts = TokenStream::from(unsafe_fn);
    assert_snapshot!(ts, @"unsafe fn() -> i32");

    let extern_fn = BareFn::new(vec![], vec![], Type::i32(), Some("C".to_string()), false);
    let ts = TokenStream::from(extern_fn);
    assert_snapshot!(ts, @"extern \"C\" fn() -> i32");
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

#[test]
fn test_attrargs_to_tokenstream() {
    let empty_args = AttrArgs::Empty;
    let ts = TokenStream::from(empty_args);
    assert_snapshot!(ts, @"");

    let delim_args =
        AttrArgs::Delimited(DelimArgs::parenthesis(TokenStream::from(vec![Token::lit(
            "test",
        )])));
    let ts = TokenStream::from(delim_args);
    assert_snapshot!(ts, @"(\"test\")");

    let eq_args = AttrArgs::Eq(Expr::new(Lit::str("value")));
    let ts = TokenStream::from(eq_args);
    assert_snapshot!(ts, @"\"value\" =");
}

#[test]
fn test_attribute_to_tokenstream() {
    let attr = Attribute::normal(AttributeItem::simple(Path::single("derive")));
    let ts = TokenStream::from(attr);
    assert_snapshot!(ts, @"#[derive]");
}

#[test]
fn test_attrkind_to_tokenstream() {
    let normal_attr = AttrKind::Normal(AttributeItem::simple(Path::single("test")));
    let ts = TokenStream::from(normal_attr);
    assert_snapshot!(ts, @"#[test]");

    let doc_comment = AttrKind::DocComment("This is a doc comment".to_string());
    let ts = TokenStream::from(doc_comment);
    assert_snapshot!(ts, @"This is a doc comment");
}

#[test]
fn test_attributeitem_to_tokenstream() {
    let simple_attr = AttributeItem::simple(Path::single("derive"));
    let ts = TokenStream::from(simple_attr);
    assert_snapshot!(ts, @"#[derive]");

    let cfg_attr = AttributeItem::cfg_feature("serde");
    let ts = TokenStream::from(cfg_attr);
    assert_snapshot!(ts, @r#"#[cfg(feature = "serde")]"#);
}

#[test]
fn test_expr_to_tokenstream() {
    let expr = Expr::new(Lit::int("42"));
    let ts = TokenStream::from(expr);
    assert_snapshot!(ts, @"42");

    let mut expr_with_attrs = Expr::new(Lit::int("42"));
    expr_with_attrs.add_attr(AttributeItem::simple(Path::single("inline")));
    let ts = TokenStream::from(expr_with_attrs);
    assert_snapshot!(ts, @"#[inline] 42");
}

#[test]
fn test_const_to_tokenstream() {
    let const_expr = Const(Expr::new(Lit::int("42")));
    let ts = TokenStream::from(const_expr);
    assert_snapshot!(ts, @"42");
}

#[test]
fn test_array_to_tokenstream() {
    let empty_array = Array::unit();
    let ts = TokenStream::from(empty_array);
    assert_snapshot!(ts, @"[]");

    let int_array = Array::new(vec![
        Expr::new(Lit::int("1")),
        Expr::new(Lit::int("2")),
        Expr::new(Lit::int("3")),
    ]);
    let ts = TokenStream::from(int_array);
    assert_snapshot!(ts, @"[1, 2, 3]");
}

#[test]
fn test_tuple_to_tokenstream() {
    let empty_tuple = Tuple::unit();
    let ts = TokenStream::from(empty_tuple);
    assert_snapshot!(ts, @"()");

    let int_tuple = Tuple::new(vec![Expr::new(Lit::int("1")), Expr::new(Lit::int("2"))]);
    let ts = TokenStream::from(int_tuple);
    assert_snapshot!(ts, @"(1, 2)");
}

#[test]
fn test_binary_to_tokenstream() {
    let binary = Binary::new(Lit::int("1"), BinOpKind::Add, Lit::int("2"));
    let ts = TokenStream::from(binary);
    assert_snapshot!(ts, @"1 + 2");

    let nested_binary = Binary::new(
        Binary::new(Lit::int("1"), BinOpKind::Add, Lit::int("2")),
        BinOpKind::Mul,
        Lit::int("3"),
    );
    let ts = TokenStream::from(nested_binary);
    assert_snapshot!(ts, @"(1 + 2) * 3");
}

#[test]
fn test_unary_to_tokenstream() {
    let unary = Unary::new(UnaryOpKind::Neg, Lit::int("42"));
    let ts = TokenStream::from(unary);
    assert_snapshot!(ts, @"-42");

    let nested_unary = Unary::new(
        UnaryOpKind::Not,
        Unary::new(UnaryOpKind::Deref, Path::single("x")),
    );
    let ts = TokenStream::from(nested_unary);
    assert_snapshot!(ts, @"!*x");
}

#[test]
fn test_let_to_tokenstream() {
    let let_stmt = Let::new(Pat::Wild, Lit::int("42"));
    let ts = TokenStream::from(let_stmt);
    assert_snapshot!(ts, @"let _ = 42");
}

#[test]
fn test_if_to_tokenstream() {
    let if_stmt = If::new(
        Expr::new(Lit::bool("true")),
        Block::from(Stmt::Expr(Expr::new(Lit::int("1")))),
        None,
    );
    let ts = TokenStream::from(if_stmt);
    assert_snapshot!(ts, @"if true { 1 }");

    let if_else_stmt = If::new(
        Expr::new(Lit::bool("false")),
        Block::from(Stmt::Expr(Expr::new(Lit::int("1")))),
        Some(Expr::from(Block::from(Stmt::Expr(Expr::new(Lit::int(
            "2",
        )))))),
    );
    let ts = TokenStream::from(if_else_stmt);
    assert_snapshot!(ts, @"if false { 1 } else { { 2 } }");
}

#[test]
fn test_while_to_tokenstream() {
    let while_stmt = While::new(
        Expr::new(Lit::bool("true")),
        Block::from(Stmt::Expr(Expr::new(Lit::int("1")))),
    );
    let ts = TokenStream::from(while_stmt);
    assert_snapshot!(ts, @"while true { 1 }");
}

#[test]
fn test_forloop_to_tokenstream() {
    let for_stmt = ForLoop::new(
        Pat::Wild,
        Path::single("items"),
        Block::from(Stmt::Expr(Expr::new(Lit::int("1")))),
    );
    let ts = TokenStream::from(for_stmt);
    assert_snapshot!(ts, @"for _ in items { 1 }");
}

#[test]
fn test_loop_to_tokenstream() {
    let loop_stmt = Loop::new(Block::from(Stmt::Expr(Expr::new(Lit::int("1")))));
    let ts = TokenStream::from(loop_stmt);
    assert_snapshot!(ts, @"loop { 1 }");
}

#[test]
fn test_constblock_to_tokenstream() {
    let const_block = ConstBlock::new(Block::from(Stmt::Expr(Expr::new(Lit::int("42")))));
    let ts = TokenStream::from(const_block);
    assert_snapshot!(ts, @"const { 42 }");
}

#[test]
fn test_unsafeblock_to_tokenstream() {
    let unsafe_block = UnsafeBlock::new(Block::from(Stmt::Expr(Expr::new(Lit::int("42")))));
    let ts = TokenStream::from(unsafe_block);
    assert_snapshot!(ts, @"unsafe { 42 }");
}

#[test]
fn test_arm_to_tokenstream() {
    let arm = Arm::new(Pat::Wild, None, Lit::int("42"));
    let ts = TokenStream::from(arm);
    assert_snapshot!(ts, @"_ => 42");

    let arm_with_guard = Arm::new(Pat::Wild, Some(Expr::new(Lit::bool("true"))), Lit::int("1"));
    let ts = TokenStream::from(arm_with_guard);
    assert_snapshot!(ts, @"_ if true => 1");
}

#[test]
fn test_match_to_tokenstream() {
    let match_expr = Match::new(
        Path::single("x"),
        vec![Arm::new(Pat::Wild, None, Lit::int("42"))],
    );
    let ts = TokenStream::from(match_expr);
    assert_snapshot!(ts, @"match x { _ => 42, }");
}

#[test]
fn test_closure_to_tokenstream() {
    let closure = Closure::simple(FnDecl::regular(vec![], None), Lit::int("42"));
    let ts = TokenStream::from(closure);
    assert_snapshot!(ts, @"| | -> { 42 }");
}

#[test]
fn test_async_to_tokenstream() {
    let async_block = Async::new(Block::from(Stmt::Expr(Expr::new(Lit::int("42")))));
    let ts = TokenStream::from(async_block);
    assert_snapshot!(ts, @"async { 42 }");
}

#[test]
fn test_await_to_tokenstream() {
    let await_expr = Await::new(Path::single("future"));
    let ts = TokenStream::from(await_expr);
    assert_snapshot!(ts, @"future.await");
}

#[test]
fn test_tryblock_to_tokenstream() {
    let try_block = TryBlock::new(Block::from(Stmt::Expr(Expr::new(Lit::int("42")))));
    let ts = TokenStream::from(try_block);
    assert_snapshot!(ts, @"try { 42 }");
}

#[test]
fn test_field_to_tokenstream() {
    let field = Field::new(Path::single("obj"), "field");
    let ts = TokenStream::from(field);
    assert_snapshot!(ts, @"obj.field");
}

#[test]
fn test_index_to_tokenstream() {
    let index = Index::new(Path::single("arr"), Lit::int("0"));
    let ts = TokenStream::from(index);
    assert_snapshot!(ts, @"arr[0]");
}

#[test]
fn test_range_to_tokenstream() {
    let half_open_range = Range::new(
        Some(Expr::new(Lit::int("0"))),
        Some(Expr::new(Lit::int("10"))),
        RangeLimits::HalfOpen,
    );
    let ts = TokenStream::from(half_open_range);
    assert_snapshot!(ts, @"0..10");

    let closed_range = Range::new(
        Some(Expr::new(Lit::int("0"))),
        Some(Expr::new(Lit::int("10"))),
        RangeLimits::Closed,
    );
    let ts = TokenStream::from(closed_range);
    assert_snapshot!(ts, @"0..=10");

    let open_range = Range::new(None, None, RangeLimits::HalfOpen);
    let ts = TokenStream::from(open_range);
    assert_snapshot!(ts, @"..");
}

#[test]
fn test_underscore_to_tokenstream() {
    let underscore = Underscore {};
    let ts = TokenStream::from(underscore);
    assert_snapshot!(ts, @"_");
}

#[test]
fn test_return_to_tokenstream() {
    let return_empty = Return::new(None::<Expr>);
    let ts = TokenStream::from(return_empty);
    assert_snapshot!(ts, @"return");

    let return_value = Return::new(Some(Lit::int("42")));
    let ts = TokenStream::from(return_value);
    assert_snapshot!(ts, @"return 42");
}

#[test]
fn test_yield_to_tokenstream() {
    let yield_empty = Yield::new(None::<Expr>);
    let ts = TokenStream::from(yield_empty);
    assert_snapshot!(ts, @"yield");

    let yield_value = Yield::new(Some(Lit::int("42")));
    let ts = TokenStream::from(yield_value);
    assert_snapshot!(ts, @"yield 42");
}

#[test]
fn test_assign_to_tokenstream() {
    let assign = Assign::new(Path::single("x"), Lit::int("42"));
    let ts = TokenStream::from(assign);
    assert_snapshot!(ts, @"x = 42");
}

#[test]
fn test_binopkind_to_tokenstream() {
    let add_op = BinOpKind::Add;
    let ts = TokenStream::from(add_op);
    assert_snapshot!(ts, @"+");

    let eq_op = BinOpKind::Eq;
    let ts = TokenStream::from(eq_op);
    assert_snapshot!(ts, @"==");
}

#[test]
fn test_assignop_to_tokenstream() {
    let assign_op = AssignOp::new(Path::single("x"), BinOpKind::Add, Lit::int("5"));
    let ts = TokenStream::from(assign_op);
    assert_snapshot!(ts, @"x + 5");
}

#[test]
fn test_lit_to_tokenstream() {
    let int_lit = Lit::int("42");
    let ts = TokenStream::from(int_lit);
    assert_snapshot!(ts, @"42");

    let str_lit = Lit::str("hello");
    let ts = TokenStream::from(str_lit);
    assert_snapshot!(ts, @"\"hello\"");

    let bool_lit = Lit::bool("true");
    let ts = TokenStream::from(bool_lit);
    assert_snapshot!(ts, @"true");
}

#[test]
fn test_cast_to_tokenstream() {
    let cast = Cast::new(Lit::int("42"), Type::i32());
    let ts = TokenStream::from(cast);
    assert_snapshot!(ts, @"42 as i32");
}

#[test]
fn test_typeascription_to_tokenstream() {
    let type_ascription = TypeAscription::new(Lit::int("42"), Type::i32());
    let ts = TokenStream::from(type_ascription);
    assert_snapshot!(ts, @"42: i32");
}

#[test]
fn test_call_to_tokenstream() {
    let call = Call::new(Path::single("func"), vec![]);
    let ts = TokenStream::from(call);
    assert_snapshot!(ts, @"func()");

    let call_with_args = Call::new(
        Path::single("func"),
        vec![Expr::new(Lit::int("1")), Expr::new(Lit::int("2"))],
    );
    let ts = TokenStream::from(call_with_args);
    assert_snapshot!(ts, @"func(1, 2)");
}

#[test]
fn test_methodcall_to_tokenstream() {
    let method_call = MethodCall::new(Path::single("obj"), PathSegment::simple("method"), vec![]);
    let ts = TokenStream::from(method_call);
    assert_snapshot!(ts, @"obj.method()");

    let method_call_with_args = MethodCall::new(
        Path::single("obj"),
        PathSegment::simple("method"),
        vec![Expr::new(Lit::int("1"))],
    );
    let ts = TokenStream::from(method_call_with_args);
    assert_snapshot!(ts, @"obj.method(1)");
}

#[test]
fn test_path_to_tokenstream() {
    let simple_path = Path::single("foo");
    let ts = TokenStream::from(simple_path);
    assert_snapshot!(ts, @"foo");

    let nested_path = Path::single("std").chain("collections").chain("HashMap");
    let ts = TokenStream::from(nested_path);
    assert_snapshot!(ts, @"std::collections::HashMap");
}

#[test]
fn test_pathsegment_to_tokenstream() {
    let simple_segment = PathSegment::simple("foo");
    let ts = TokenStream::from(simple_segment);
    assert_snapshot!(ts, @"foo");

    let generic_segment = PathSegment::new("Vec", Some(vec![GenericArg::Type(Type::i32())]));
    let ts = TokenStream::from(generic_segment);
    assert_snapshot!(ts, @"Vec::<i32>");
}

#[test]
fn test_addrof_to_tokenstream() {
    let ref_immut = AddrOf::new(BorrowKind::Ref, Mutability::Not, Path::single("x"));
    let ts = TokenStream::from(ref_immut);
    assert_snapshot!(ts, @"&x");

    let ref_mut = AddrOf::new(BorrowKind::Ref, Mutability::Mut, Path::single("x"));
    let ts = TokenStream::from(ref_mut);
    assert_snapshot!(ts, @"&mut x");

    let raw_const = AddrOf::new(BorrowKind::Raw, Mutability::Not, Path::single("x"));
    let ts = TokenStream::from(raw_const);
    assert_snapshot!(ts, @"&raw const x");

    let raw_mut = AddrOf::new(BorrowKind::Raw, Mutability::Mut, Path::single("x"));
    let ts = TokenStream::from(raw_mut);
    assert_snapshot!(ts, @"&raw mut x");
}

#[test]
fn test_break_to_tokenstream() {
    let break_empty = Break::new(None, None);
    let ts = TokenStream::from(break_empty);
    assert_snapshot!(ts, @"break");

    let break_with_value = Break::new(None, Some(Expr::new(Lit::int("42"))));
    let ts = TokenStream::from(break_with_value);
    assert_snapshot!(ts, @"break 42");

    let break_with_label = Break::new(Some("loop1".to_string()), None);
    let ts = TokenStream::from(break_with_label);
    assert_snapshot!(ts, @"break 'loop1");
}

#[test]
fn test_continue_to_tokenstream() {
    let continue_empty = Continue::new(None);
    let ts = TokenStream::from(continue_empty);
    assert_snapshot!(ts, @"continue");

    let continue_with_label = Continue::new(Some("loop1".to_string()));
    let ts = TokenStream::from(continue_with_label);
    assert_snapshot!(ts, @"continue 'loop1");
}

#[test]
fn test_genericarg_to_tokenstream() {
    let lifetime_arg = GenericArg::Lifetime("static".to_string());
    let ts = TokenStream::from(lifetime_arg);
    assert_snapshot!(ts, @"'static");

    let type_arg = GenericArg::Type(Type::i32());
    let ts = TokenStream::from(type_arg);
    assert_snapshot!(ts, @"i32");

    let const_arg = GenericArg::Const(Const(Expr::new(Lit::int("10"))));
    let ts = TokenStream::from(const_arg);
    assert_snapshot!(ts, @"10");
}

#[test]
fn test_delimargs_to_tokenstream() {
    let paren_args = DelimArgs::parenthesis(TokenStream::from(vec![Token::lit("test")]));
    let ts = TokenStream::from(paren_args);
    assert_snapshot!(ts, @"(\"test\")");

    let bracket_args = DelimArgs::bracket(TokenStream::from(vec![Token::lit("test")]));
    let ts = TokenStream::from(bracket_args);
    assert_snapshot!(ts, @"[\"test\"]");

    let brace_args = DelimArgs::brace(TokenStream::from(vec![Token::lit("test")]));
    let ts = TokenStream::from(brace_args);
    assert_snapshot!(ts, @"{\"test\"}");
}

#[test]
fn test_maccall_to_tokenstream() {
    let mac_call = MacCall::new(
        Path::single("println"),
        DelimArgs::parenthesis(TokenStream::from(vec![Token::lit("Hello")])),
    );
    let ts = TokenStream::from(mac_call);
    assert_snapshot!(ts, @r#"println!("Hello")"#);
}

#[test]
fn test_exprfield_to_tokenstream() {
    let expr_field = ExprField::new("name", Lit::str("value"));
    let ts = TokenStream::from(expr_field);
    assert_snapshot!(ts, @"name: \"value\"");
}

#[test]
fn test_struct_to_tokenstream() {
    let struct_expr = Struct::new(
        Path::single("Person"),
        vec![
            ExprField::new("name", Lit::str("Alice")),
            ExprField::new("age", Lit::int("30")),
        ],
    );
    let ts = TokenStream::from(struct_expr);
    assert_snapshot!(ts, @r#"Person{ name: "Alice", age: 30 }"#);
}

#[test]
fn test_repeat_to_tokenstream() {
    let repeat = Repeat::new(Lit::int("0"), Const(Expr::new(Lit::int("10"))));
    let ts = TokenStream::from(repeat);
    assert_snapshot!(ts, @"[0; 10]");
}

#[test]
fn test_try_to_tokenstream() {
    let try_expr = Try::new(Path::single("result"));
    let ts = TokenStream::from(try_expr);
    assert_snapshot!(ts, @"result?");
}

#[test]
fn test_genericparam_to_tokenstream() {
    let type_param = GenericParam::TypeParam(TypeParam::simple("T"));
    let ts = TokenStream::from(type_param);
    assert_snapshot!(ts, @"T");

    let const_param = GenericParam::ConstParam(ConstParam::new("N", Type::usize()));
    let ts = TokenStream::from(const_param);
    assert_snapshot!(ts, @"const N: usize");
}

#[test]
fn test_type_ptr_variants_to_tokenstream() {
    let const_ptr_ty = Type::Ptr(Ptr::new(PtrKind::Const, Type::i32()));
    let ts = TokenStream::from(const_ptr_ty);
    assert_snapshot!(ts, @"*const i32");

    let mut_ptr_ty = Type::Ptr(Ptr::new(PtrKind::Mut, Type::str()));
    let ts = TokenStream::from(mut_ptr_ty);
    assert_snapshot!(ts, @"*mut str");
}

#[test]
fn test_type_ref_variants_to_tokenstream() {
    let ref_ty = Type::Ref(Ref::new(None::<String>, MutTy::immut(Type::bool())));
    let ts = TokenStream::from(ref_ty);
    assert_snapshot!(ts, @"&bool");

    let ref_mut_ty = Type::Ref(Ref::new(Some("static"), MutTy::mut_(Type::char())));
    let ts = TokenStream::from(ref_mut_ty);
    assert_snapshot!(ts, @"&'static mut char");
}

#[test]
fn test_type_barefn_to_tokenstream() {
    let fn_ty = Type::BareFn(BareFn::safe(vec![], vec![], Type::unit()));
    let ts = TokenStream::from(fn_ty);
    assert_snapshot!(ts, @"fn() -> ()");

    let unsafe_fn_ty = Type::BareFn(BareFn::new(
        vec![],
        vec![],
        Type::i32(),
        Some("C".to_string()),
        true,
    ));
    let ts = TokenStream::from(unsafe_fn_ty);
    assert_snapshot!(ts, @"unsafe extern \"C\" fn() -> i32");
}

#[test]
fn test_type_macro_to_tokenstream() {
    let mac_ty = Type::Macro(MacCall::new(
        Path::single("vec"),
        DelimArgs::bracket(TokenStream::from(vec![Token::lit("u8")])),
    ));
    let ts = TokenStream::from(mac_ty);
    assert_snapshot!(ts, @"vec![\"u8\"]");
}

#[test]
fn test_type_implicit_self_to_tokenstream() {
    let implicit_self_ty = Type::ImplicitSelf;
    let ts = TokenStream::from(implicit_self_ty);
    assert_snapshot!(ts, @"");
}

#[test]
fn test_type_err_to_tokenstream() {
    let err_ty = Type::Err;
    let ts = TokenStream::from(err_ty);
    assert_snapshot!(ts, @"<Err>");
}

#[test]
fn test_type_traitobject_variants_to_tokenstream() {
    let dyn_trait_ty = Type::TraitObject(TraitObject::dyn_(vec![
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Send"))),
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Sync"))),
    ]));
    let ts = TokenStream::from(dyn_trait_ty);
    assert_snapshot!(ts, @"dyn Send + Sync");

    let static_trait_ty = Type::TraitObject(TraitObject::static_(vec![GenericBound::Outlives(
        "static".to_string(),
    )]));
    let ts = TokenStream::from(static_trait_ty);
    assert_snapshot!(ts, @"'static");
}

#[test]
fn test_type_impltrait_variants_to_tokenstream() {
    let impl_trait_ty = Type::ImplTrait(ImplTrait::new(vec![
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Iterator"))),
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Clone"))),
    ]));
    let ts = TokenStream::from(impl_trait_ty);
    assert_snapshot!(ts, @"impl Iterator + Clone");
}

#[test]
fn test_complex_nested_types_to_tokenstream() {
    // Complex nested type: &'static mut [Box<dyn Send + Sync>; 10]
    let inner_trait = TraitObject::dyn_(vec![
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Send"))),
        GenericBound::Trait(PolyTraitRef::simple(Path::single("Sync"))),
    ]);
    let boxed_trait = Type::box_(Type::TraitObject(inner_trait));
    let array_ty = Type::Array(
        Box::new(boxed_trait),
        Box::new(Const(Expr::new(Lit::int("10")))),
    );
    let ref_mut_ty = Type::Ref(Ref::new(Some("static"), MutTy::mut_(array_ty)));
    let ts = TokenStream::from(ref_mut_ty);
    assert_snapshot!(ts, @"&'static mut [Box::<dyn Send + Sync>; 10]");
}

#[test]
fn test_empty_bounds_to_tokenstream() {
    let empty_trait_obj = TraitObject::static_(vec![]);
    let ts = TokenStream::from(empty_trait_obj);
    assert_snapshot!(ts, @"");

    let empty_impl_trait = ImplTrait::new(vec![]);
    let ts = TokenStream::from(empty_impl_trait);
    assert_snapshot!(ts, @"impl ");
}

#[test]
fn test_barefn_with_params_to_tokenstream() {
    let param1 = Param::ident("x", Type::i32());
    let param2 = Param::ident("y", Type::str());
    let fn_with_params = BareFn::safe(vec![], vec![param1, param2], Type::bool());
    let ts = TokenStream::from(fn_with_params);
    assert_snapshot!(ts, @"fn(x: i32, y: str) -> bool");
}

#[test]
fn test_typeparam_multiple_bounds_to_tokenstream() {
    let multi_bound_param = TypeParam::new(
        "T",
        vec![
            GenericBound::Trait(PolyTraitRef::simple(Path::single("Clone"))),
            GenericBound::Trait(PolyTraitRef::simple(Path::single("Debug"))),
            GenericBound::Outlives("static".to_string()),
        ],
    );
    let ts = TokenStream::from(multi_bound_param);
    assert_snapshot!(ts, @"T: Clone + Debug + 'static");
}
