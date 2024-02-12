from extism import host_fn, Plugin, set_log_file
import sys
import json

set_log_file("stdout", level='trace')

@host_fn()
def myHostFunction1(input: str) -> str:
    print("Got input myHostFunction1: " + input)
    return "myHostFunction1: " + input

@host_fn()
def myHostFunction2(input: str) -> str:
    myobj = json.loads(input)
    print("Got input myHostFunction2: " + str(myobj))
    myobj["hello"] = "myHostFunction2"
    return json.dumps(myobj)

with Plugin(open(sys.argv[1], "rb").read(), wasi=True) as plugin:
    result = plugin.call("greet", b"Benjamin")
    print(result)

