from extism import host_fn, Plugin, set_log_file

set_log_file("stdout", level='info')

@host_fn()
def myHostFunction1(input: str) -> str:
    print("Hello from myHostFunction1 in Python!")
    print(input)
    return "Hello from Python!"

@host_fn()
def myHostFunction2(input: str) -> str:
    print("Hello from myHostFunction2 in Python!")
    print(input)
    return "Hello from Python!"

with Plugin(open("plugin.wasm", "rb").read(), wasi=True) as plugin:
    result = plugin.call("greet", b"Benjamin")
    print(result)

