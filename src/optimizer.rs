use syn::*;
use syn::visit_mut::VisitMut;

pub fn optimize(file: &mut File) {
	let mut opt = Optimizer {
		attributes_removed: 0,
		bound_checks_removed: 0,
		debug_asserts_removed: 0,
	};
	opt.visit_file_mut(file);
	eprintln!("{:#?}", opt);
}

#[derive(Debug)]
struct Optimizer {
	attributes_removed: u64,
	bound_checks_removed: u64,
	debug_asserts_removed: u64,
}

impl VisitMut for Optimizer {
	fn visit_block_mut(&mut self, i: &mut Block) {
		i.stmts.retain(|s| !self.filter_out_stmt(s));
		for it in &mut i.stmts {
		   self.visit_stmt_mut(it)
		}
	}

	fn visit_expr_assign_mut(&mut self, e: &mut ExprAssign) {
		self.optimize_lvalue(&mut e.left, true);
	}

	fn visit_expr_assign_op_mut(&mut self, e: &mut ExprAssignOp) {
		self.optimize_lvalue(&mut e.left, true);
	}

	fn visit_file_mut(&mut self, f: &mut File) {
		f.attrs.retain(|a| self.retain_attribute(&a));
		
		for i in &mut f.items {
			self.visit_item_mut(i);
		}
	}

	fn visit_item_fn_mut(&mut self, fun: &mut ItemFn) {
		fun.attrs.retain(|a| self.retain_attribute(&a));
		self.visit_block_mut(&mut fun.block);
	}
}

impl Optimizer {
	fn is_debug_assert(&mut self, path: &syn::Path) -> bool {
		if path.is_ident("debug_assert") || path.is_ident("debug_assert_eq") {
			self.debug_asserts_removed += 1;
			true
		} else {
			false
		}
	}
	
	fn filter_out_stmt(&mut self, s: &Stmt) -> bool {
		match s {
			Stmt::Expr(Expr::Macro(mac)) => {
				if self.is_debug_assert(&mac.mac.path) {
					return true;
				}
			}
			Stmt::Semi(Expr::Macro(mac), _) => {
				if self.is_debug_assert(&mac.mac.path) {
					return true;
				}
			}
			_ => {}
		}
		false
	}
	
	fn optimize_lvalue(&mut self, e: &mut Expr, outer: bool) {
		match e {
			// Replace slice indexing with get_unchecked_mut
			Expr::Index(ref mut ei) => {
				self.bound_checks_removed += 1;
				
				// Recurse to optimize the `e[i1]` in `e[i1][i2]`
				self.optimize_lvalue(&mut ei.expr, false);

				// Get hold of a span for synthetic tokens.
				let span = ei.bracket_token.span;

				// Replace `e[i]` with `e.get_unchecked_mut(i)`
				*e = Expr::MethodCall(ExprMethodCall {
					attrs: vec![],
					receiver: Box::new(*ei.expr.clone()),
					dot_token: syn::token::Dot {
						spans: [span],
					},
					method: Ident::new("get_unchecked_mut", span),
					turbofish: None,
					paren_token: syn::token::Paren {
						span
					},
					args: {
						let mut args = syn::punctuated::Punctuated::new();
						args.push_value(*ei.index.clone());
						args
					}
				});

				// If we are at the outmost level, wrap the whole in a `*unsafe { ... }` expression.
				if outer {
					*e = Expr::Unary(ExprUnary {
						attrs: vec![],
						op: syn::UnOp::Deref(syn::token::Star {
							spans: [span],
						}),
						expr: Box::new(Expr::Unsafe(ExprUnsafe {
							attrs: vec![],
							unsafe_token: syn::token::Unsafe {
								span
							},
							block: Block {
								brace_token: syn::token::Brace {
									span,
								},
								stmts: vec![Stmt::Expr(e.clone())],
							}
						}))
					});
				}
			}

			// All other expressions left untouched.
			_ => {}
		}
	}
	
	fn retain_attribute(&mut self, a: &Attribute) -> bool {
		if a.path.is_ident("doc") || a.path.is_ident("allow") {
			self.attributes_removed += 1;
			return false;
		}
		
		true
	}	
}
