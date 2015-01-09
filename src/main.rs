use std::collections::HashMap;

#[derive(Show, PartialEq)]
struct ValueKnowledge;
#[derive(Show, PartialEq)]
struct CodeReference;

#[derive(PartialEq, Clone, Show)]
enum Type {
	Unknown(u32),
	Record(Option<String>, Vec<Type>),
}

#[derive(Show, PartialEq)]
struct Value {
	typ:  Type,
	value: ValueKnowledge,
	pos: CodeReference
}

#[derive(Show, PartialEq)]
enum Goto {
	Branch(usize, usize, usize),	// discriminator, alt1, alt2
	Goto(u32),
	DeadEnd
}

#[derive(Show, PartialEq)]
enum Operation {
	Variable(String),
	MethodCall(String, Vec<u32>),
	Call(String, Vec<u32>),
	Tuple(Vec<usize>),
	Ref(u32),
	Unpack(u32),
	Write(u32),
	Read(u32, String),	// ref, fieldname (can only be used on ReadMany types)
	Return(u32)
}

#[derive(Show, PartialEq)]
struct BBlock {
	vars: HashMap<String, usize>,
	ops: Vec<(usize, Operation)>,
	goto: Goto
}

#[derive(Show, PartialEq)]
struct Function {
	blocks: Vec<BBlock>,
	values: Vec<Value>,
	type_params: Vec<String>
}

#[derive(Show)]
struct TypeError;

fn infer_forward(func: &mut Function) -> Result<(), TypeError> {
	use Operation::*;
	use Type::*;
	for block in func.blocks.iter() {
		for &(pos, ref op) in block.ops.iter() {
			match op {
				&Tuple(ref content) => {
					func.values[pos].typ = Record(None, match func.values[pos].typ {
						Record(None, ref cont) => content.iter().zip(cont.iter()).map(
							|&: (a, b)| superimpose(b, &func.values[*a].typ).unwrap()
						).collect(),
						Unknown(_) => content.iter().map(
							|&: a| func.values[*a].typ.clone() ).collect(),
						_ => return Err(TypeError)
					});
				}
				&Return(_) => {}
				_ => unimplemented!()
			}
		}
	}
	Ok(())
}

fn superimpose(a: &Type, b: &Type) -> Result<Type, TypeError> {
	use Type::*;
	match (a, b) {
		(&Unknown(_), x) => Ok(x.clone()),
		(x, &Unknown(_)) => Ok(x.clone()),
		_ => if a == b {
			Ok(b.clone())
		} else {
			Err(TypeError)
		}
	}
}

fn infer_backward(func: &mut Function) -> Result<(), TypeError> {
	use Operation::*;
	use Type::*;
	for block in func.blocks.iter() {
		for &(pos, ref op) in block.ops.iter().rev() {
			match op {
				&Tuple(ref content) => {
					let cpy = match func.values[pos].typ {
						Record(None, ref x) => x.clone(),
						_ => panic!()
					};
					for (&idx, typ) in content.iter().zip(cpy.into_iter()) {
						func.values[idx].typ = superimpose(&func.values[idx].typ, &typ).unwrap();
					}
				}
				&Return(_) => {}
				_ => unimplemented!()
			}
		}
		// TODO: infer variables across blocks
	}
	Ok(())
}

fn type_inference(mut func: Function) -> Function {
	infer_forward(&mut func).unwrap();
	infer_backward(&mut func).unwrap();
	println!("{:?}", func);
	func
}

// single basic block functions
#[test]
fn emptyfunc() {
	use Operation::*;
	use Type::*;
	assert_eq!(type_inference(Function {
		blocks: vec![ BBlock {
			vars: HashMap::new(),	// no parameters
			ops: vec![ (0, Tuple(Vec::new())),
				(1, Return(0)) ],	// return ()
			goto: Goto::DeadEnd
		} ],
		values: vec![ Value {
			typ: Unknown(1),	// unit return value (inferred)
			value: ValueKnowledge,
			pos: CodeReference
		} ],
		type_params: Vec::new()
	}), Function { blocks: vec![BBlock { vars: HashMap::new(), ops: vec![(0, Tuple(Vec::new())), (1, Return(0))], goto: Goto::DeadEnd }], values: vec![Value { typ: Record(None, vec![]), value: ValueKnowledge, pos: CodeReference }], type_params: Vec::new() });
}

#[test]
fn dropfunc() {
	use Operation::*;
	use Type::*;
	let mut param = HashMap::with_capacity(1);
	param.insert("param".to_string(), 0);
	assert_eq!(type_inference(Function {
		blocks: vec![ BBlock {
			vars: param.clone(),
			ops: vec![ (0, Tuple(Vec::new())),
				(1, Return(0)) ],	// return ()
			goto: Goto::DeadEnd
		} ],
		values: vec![ Value {
			typ: Unknown(2),	// unit return value (inferred)
			value: ValueKnowledge,
			pos: CodeReference
		} ],
		type_params: vec![ "Drop".to_string() ],
	}), Function { blocks: vec![BBlock { vars: param, ops: vec![(0u, Tuple(Vec::new())), (1u, Return(0u32))], goto: DeadEnd }], values: vec![Value { typ: Record(None, Vec::new()), value: ValueKnowledge, pos: CodeReference }], type_params: vec!["Drop"] });
}