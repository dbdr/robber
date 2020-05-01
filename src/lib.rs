use cargo_metadata::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use proc_macro2::TokenStream;

pub fn bundle<P: AsRef<Path>>(crate_dir: P) -> String {
	let manifest = crate_dir.as_ref().join("Cargo.toml");
	let mut cmd = MetadataCommand::new();
	let cmd = cmd.manifest_path(manifest);
	let metadata = cmd.exec().unwrap();
	//eprintln!("{:#?}", metadata);
	let root = metadata.resolve.as_ref().expect("dependency resolution failed").root.as_ref().unwrap_or(
		&metadata.packages.iter().find(|p| p.targets.iter().any(|t| t.kind.iter().any(|k| k == "bin"))).expect("bin package not found").id);
	let mut bundler = Bundler::new();
	bundler.bundle_package(root, &metadata, true);
	bundler.output
}

struct Bundler {
	output: String,
	bundled: HashSet<String>,
}

impl Bundler {
	fn new() -> Self {
		Self {
			output: String::new(),
			bundled: HashSet::new(),
		}
	}
	
	fn bundle_package(&mut self, pkg: &PackageId, metadata: &Metadata, root: bool) {
		eprintln!("Bundling {}", pkg);
		let node = metadata.resolve.as_ref().unwrap().nodes.iter().find(|n| n.id == *pkg).unwrap_or_else(|| panic!("package {} not found in resolve", pkg));
		for dep in &node.deps {
			if Self::is_builtin(dep) {
				continue;
			}
			if self.bundled.contains(&dep.pkg.repr) {
				continue;
			}
			self.bundled.insert(dep.pkg.repr.clone());
			eprintln!("{} depends on {}", pkg, dep.pkg);
			self.bundle_package(&dep.pkg, metadata, false);
		}
		if let Some(lib) = &metadata[pkg].targets.iter().find(|t| t.kind.iter().any(|k| k == "lib")) {
			self.output.push_str(&format!("pub mod {} {{\nuse super::*;\n", lib.name.replace('-', "_")));
			self.process_source(&lib.src_path);
			self.output.push_str("\n}\n");
		}
		if root {
			let bin = &metadata[pkg].targets.iter().find(|t| /*t.name == "player" &&*/ t.kind.iter().any(|k| k == "bin")).unwrap().src_path;
			self.process_source(bin);
		}
	}

	fn is_builtin(dep: &NodeDep) -> bool {
		//FIXME
		dep.name == "rand" || dep.name == "log" || dep.name == "env_logger"
	}

	fn process_source(&mut self, src: &PathBuf) {
		eprintln!("{:?}", src);
		let code = read_file(&Path::new(src));
		let mut file = syn::parse_file(&code).unwrap_or_else(|e| panic!("failed to parse {:?}: {}", src, e));

		let base_path = Path::new(src).parent().expect("lib.src_path has no parent").to_path_buf();

		Expander {
			base_path: base_path.clone(),
		}.visit_file_mut(&mut file);

		let tokens = process_include_str(file.into_token_stream(), &base_path);

		use quote::ToTokens;
		self.output.push_str(&format!("{}", tokens));
	}
}

fn process_include_str(source: TokenStream, base_path: &PathBuf) -> TokenStream {
	let mut tokens = Vec::<proc_macro2::TokenTree>::new();
	let mut source = source.into_iter();
	while let Some(token) = source.next() {
		use proc_macro2::TokenTree::*;
		match &token {
			Ident(i) => if i == "include_str" {
				let next = source.next();
				if let Some(Punct(_)) = next {
					// include_str!
					let next = source.next();
					if let Some(Group(g)) = next {
						let filename = g.stream().into_iter().next().unwrap();
						//FIXME
						let filename = filename.to_string().trim_matches('"').to_string();
						let path = base_path.join(&filename);
						let contents = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Included file {:?} not found: {}", path, e));
						eprintln!("Processing {} {} => {:?}", i, filename, contents);
						tokens.push(Literal(proc_macro2::Literal::string(&contents)));
						continue;
					}
					panic!("Unexpected, the token after include_str! is {:?}", next);
				} else {
					tokens.push(Ident(proc_macro2::Ident::new(&i.to_string(), i.span())));
					if let Some(next) = next {
						tokens.push(next);
					}
				}
			}
			Group(g) => {
				tokens.push(Group(proc_macro2::Group::new(g.delimiter(), process_include_str(g.stream(), base_path))));
				continue;
			}
			_ => {}
		}
		tokens.push(token);
	}
	use std::iter::FromIterator;
	proc_macro2::TokenStream::from_iter(tokens.into_iter())
}

struct Expander {
    base_path: PathBuf,
}

use syn::visit_mut::VisitMut;
impl VisitMut for Expander {
	fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
		for it in &mut item.attrs {
			self.visit_attribute_mut(it)
		}
		self.visit_visibility_mut(&mut item.vis);
		self.visit_ident_mut(&mut item.ident);
		self.expand_mods(item);
		if let Some(ref mut it) = item.content {
			for it in &mut (it).1 {
				self.visit_item_mut(it);
			}
		}
	}

	/* This works, but I don't see how to replace it with the contents. Probably not the right approach.
	fn visit_expr_macro_mut(&mut self, item: &mut syn::ExprMacro) {
		if item.mac.path.is_ident("include_str") {
			let filename = item.mac.tts.to_string().trim_matches('"').to_string();
			let path = self.base_path.join(&filename);
			let contents = std::fs::read_to_string(&path).expect(&format!("Included file {:?} not found", path));
			eprintln!("Including {} as {:?}", filename, contents);
		}
	}
	*/
}

impl Expander {
	fn expand_mods(&self, item: &mut syn::ItemMod) {
		let name = item.ident.to_string();
		if item.content.is_some() {
			//TODO check attrs; remove whole #[cfg(test)] modules and not just #[test] functions
			if name == "tests" {
				item.content = Some((Default::default(), Vec::new()));
			}
			return;
		}
		let other_base_path = self.base_path.join(&name);
		let (base_path, code) = vec![
			(self.base_path.clone(), format!("{}.rs", name)),
			(other_base_path, String::from("mod.rs")),
		].into_iter()
			.map(|(base_path, file_name)| {
				(base_path.clone(), read_file(&base_path.join(file_name)))
			})
		.next()
			.expect("mod not found");
		eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
		let mut file = syn::parse_file(&code).expect("failed to parse file");
		Expander {
			base_path,
		}.visit_file_mut(&mut file);
		item.content = Some((Default::default(), file.items));
	}
}

fn read_file(path: &Path) -> String {
	let mut buf = String::new();
	use std::io::Read;
	File::open(path).unwrap_or_else(|e| panic!("failed to open {:?}: {}", path, e)).read_to_string(&mut buf).unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
	buf
}
