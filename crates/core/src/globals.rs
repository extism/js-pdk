use anyhow::anyhow;
use extism_pdk::bindings::{extism_load_input, extism_log_info};
use extism_pdk::*;
use quickjs_wasm_rs::{Context, Value};
use std::borrow::Cow;
use std::io::{Read, Write};
use std::str;

pub fn inject_globals(
    context: &Context,
) -> anyhow::Result<()>
{
    let global = context.global_object()?;

    // TODO these should proxy to extism_pdk's log functions
    // let console_log_callback = context.wrap_callback(console_log_to(log_stream))?;
    // let console_error_callback = context.wrap_callback(console_log_to(error_stream))?;
    // let console_object = context.object_value()?;
    // console_object.set_property("log", console_log_callback)?;
    // console_object.set_property("error", console_error_callback)?;
    // global.set_property("console", console_object)?;

    let module_obj = context.object_value()?;
    module_obj.set_property("exports", context.null_value()?)?;
    global.set_property("module", module_obj)?;

    // Extism Host object
    let host_object = context.object_value()?;
    let host_input_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            ctx.array_buffer_value(&input)
        })?;
    host_object.set_property("inputBytes", host_input_bytes)?;
    let host_input_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let input = unsafe { extism_load_input() };
            let string = String::from_utf8(input)?;
            ctx.value_from_str(&string)
        })?;
    host_object.set_property("inputString", host_input_string)?;

    let host_output_bytes =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_bytes()?)?;
            ctx.value_from_bool(true)
        })?;
    host_object.set_property("outputBytes", host_output_bytes)?;

    let host_output_string =
        context.wrap_callback(|ctx: &Context, _this: &Value, args: &[Value]| {
            let output = args.get(0).unwrap();
            extism_pdk::output(output.as_str()?)?;
            ctx.value_from_bool(true)
        })?;
    host_object.set_property("outputString", host_output_string)?;

    global.set_property("Host", host_object)?;

    Ok(())
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<Value>
where
    T: Write + 'static,
{
    move |ctx: &Context, _this: &Value, args: &[Value]| {
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                write!(stream, " ")?;
            }

            stream.write_all(arg.as_str()?.as_bytes())?;
        }

        writeln!(stream)?;
        ctx.undefined_value()
    }
}

// fn encode_js_string_to_utf8_buffer(
// ) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<Value> {
//     move |ctx: &Context, _this: &Value, args: &[Value]| {
//         if args.len() != 1 {
//             return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
//         }

//         let js_string = args[0].as_str_lossy();
//         ctx.array_buffer_value(js_string.as_bytes())
//     }
// }

// fn js_args_to_io_writer(args: &[Value]) -> anyhow::Result<(Box<dyn Write>, &[u8])> {
//     // TODO: Should throw an exception
//     let [fd, data, offset, length, ..] = args else {
//         anyhow::bail!("Invalid number of parameters");
//     };

//     let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
//     let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

//     let fd: Box<dyn Write> = match fd.try_as_integer()? {
//         1 => Box::new(std::io::stdout()),
//         2 => Box::new(std::io::stderr()),
//         _ => anyhow::bail!("Only stdout and stderr are supported"),
//     };

//     if !data.is_array_buffer() {
//         anyhow::bail!("Data needs to be an ArrayBuffer");
//     }
//     let data = data.as_bytes()?;
//     Ok((fd, &data[offset..(offset + length)]))
// }

// fn js_args_to_io_reader(args: &[Value]) -> anyhow::Result<(Box<dyn Read>, &mut [u8])> {
//     // TODO: Should throw an exception
//     let [fd, data, offset, length, ..] = args else {
//         anyhow::bail!("Invalid number of parameters");
//     };

//     let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
//     let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

//     let fd: Box<dyn Read> = match fd.try_as_integer()? {
//         0 => Box::new(std::io::stdin()),
//         _ => anyhow::bail!("Only stdin is supported"),
//     };

//     if !data.is_array_buffer() {
//         anyhow::bail!("Data needs to be an ArrayBuffer");
//     }
//     let data = data.as_bytes_mut()?;
//     Ok((fd, &mut data[offset..(offset + length)]))
// }

// #[cfg(test)]
// mod tests {
//     use super::inject_javy_globals;
//     use anyhow::Result;
//     use quickjs_wasm_rs::Context;
//     use std::cell::RefCell;
//     use std::io;
//     use std::rc::Rc;

//     #[test]
//     fn test_console_log() -> Result<()> {
//         let mut stream = SharedStream::default();

//         let ctx = Context::default();
//         inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

//         ctx.eval_global("main", "console.log(\"hello world\");")?;
//         assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

//         stream.clear();

//         ctx.eval_global("main", "console.log(\"bonjour\", \"le\", \"monde\")")?;
//         assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());

//         stream.clear();

//         ctx.eval_global(
//             "main",
//             "console.log(2.3, true, { foo: 'bar' }, null, undefined)",
//         )?;
//         assert_eq!(
//             b"2.3 true [object Object] null undefined\n",
//             stream.0.borrow().as_slice()
//         );
//         Ok(())
//     }

//     #[test]
//     fn test_console_error() -> Result<()> {
//         let mut stream = SharedStream::default();

//         let ctx = Context::default();
//         inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

//         ctx.eval_global("main", "console.error(\"hello world\");")?;
//         assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

//         stream.clear();

//         ctx.eval_global("main", "console.error(\"bonjour\", \"le\", \"monde\")")?;
//         assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());

//         stream.clear();

//         ctx.eval_global(
//             "main",
//             "console.error(2.3, true, { foo: 'bar' }, null, undefined)",
//         )?;
//         assert_eq!(
//             b"2.3 true [object Object] null undefined\n",
//             stream.0.borrow().as_slice()
//         );
//         Ok(())
//     }

//     #[derive(Default, Clone)]
//     struct SharedStream(Rc<RefCell<Vec<u8>>>);

//     impl SharedStream {
//         fn clear(&mut self) {
//             (*self.0).borrow_mut().clear();
//         }
//     }

//     impl io::Write for SharedStream {
//         fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//             (*self.0).borrow_mut().write(buf)
//         }

//         fn flush(&mut self) -> io::Result<()> {
//             (*self.0).borrow_mut().flush()
//         }
//     }
// }
