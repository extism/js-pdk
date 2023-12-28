from extism import host_fn, Plugin, set_log_file
import sys

set_log_file("stdout", level='info')

@host_fn()
def myHostFunction1(input: str) -> str:
    print("Got input myHostFunction2: " + input)
    return input

@host_fn()
def myHostFunction2(input: str) -> str:
    print("Got input myHostFunction2: " + input)
    return input

with Plugin(open(sys.argv[1], "rb").read(), wasi=True) as plugin:
    result = plugin.call("greet", b"Benjamin")
    print(result)

