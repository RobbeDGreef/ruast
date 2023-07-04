fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rast::*;

    let mut krate = Crate::new();
    let def = Fn::main(
        None,
        Block::from(Path::single("println").mac_call(vec![Token::lit("Hello, world!")])),
    );
    krate.add_item(def);
    println!("{krate}");
    // krate.dump("hello.rs")?;
    Ok(())
}
