fn main() -> Result<(), String> {
    let facts = algorithm_shapes_explainer::facts::facts()?;
    facts.self_check()?;
    print!("{}", facts.print());
    Ok(())
}
