#[test]
fn bundle() -> Result<(), Box<dyn std::error::Error>> {
	use std::fs::*;
	use std::process::Command;
	
	for bot in read_dir("tests/input")? {
		let bot = bot?.path();
		let bot_name = bot.file_name().unwrap().to_str().unwrap();
		eprintln!("\n\n### {:?}", bot);
		let source = robber::bundle(&bot);
		let source_file  = format!("generated/{}_bot.rs", bot_name);
		let compiled_bot = format!("generated/{}_bot", bot_name);
		std::fs::write(&source_file, source)?;
		
		let rustc_output = Command::new("rustc").args(&[&source_file, "-o", &compiled_bot]).output()?;
		if ! rustc_output.status.success() {
			panic!("Compilation of bundled version of {:?} failed:\n{}", bot, String::from_utf8(rustc_output.stderr)?);
		}
		
		let bot_output = Command::new(&compiled_bot).output()?;
		assert!(bot_output.status.success());
		assert_eq!("Hello, CodinGame!\n", String::from_utf8(bot_output.stdout)?);
		std::fs::remove_file(compiled_bot)?;
	}
	Ok(())
}
