package main

import (
	"context"
	"fmt"
	"math"
	"os"
	"strings"

	extism "github.com/extism/go-sdk"
)

// Capitalize a string (Host function for `capitalize`)
func capitalize(ctx context.Context, p *extism.CurrentPlugin, stack []uint64) {
	input, err := p.ReadString(stack[0])
	if err != nil {
		panic(err)
	}

	capitalized := strings.Title(input)
	s, err := p.WriteString(capitalized)
	if err != nil {
		panic(err)
	}

	stack[0] = s
}

// Handle floating point inputs (Host function for `floatInputs`)
func floatInputs(ctx context.Context, p *extism.CurrentPlugin, stack []uint64) {
	f64 := extism.DecodeF64(stack[0])
	f32 := float32(extism.DecodeF32(stack[1]))

	fmt.Println("Go Host: floatInputs received:", f64, f32)

	// Return a fixed integer (expected behavior based on JavaScript code)
	stack[0] = uint64(2_147_483_647) // Max int32 value
}

// Handle integer input, return a floating point (Host function for `floatOutput`)
func floatOutput(ctx context.Context, p *extism.CurrentPlugin, stack []uint64) {
	i32 := int32(stack[0])

	fmt.Println("Go Host: floatOutput received:", i32)

	// Return the expected float value
	result := 9_007_199_254_740.125
	stack[0] = extism.EncodeF64(result)
}

// Handle multiple arguments but return nothing (Host function for `voidInputs`)
func voidInputs(ctx context.Context, p *extism.CurrentPlugin, stack []uint64) {
	fmt.Printf("Go Host: voidInputs stack: %v, %v, %v, %v, %v\n", stack[0], stack[1], stack[2], stack[3], stack[4])

	i32 := int32(stack[0])
	i64 := int64(stack[1])
	f32 := float32(extism.DecodeF32(stack[2]))
	f64 := extism.DecodeF64(stack[3])
	extra := int32(stack[4])

	fmt.Printf("Go Host: voidInputs received: i32=%d, i64=%d, f32=%f, f64=%f, extra=%d\n", i32, i64, f32, f64, extra)

	if i32 != 2_147_483_647 {
		panic("Unexpected i32 value: " + fmt.Sprint(i32))
	}

	if i64 != 9_223_372_036_854_775_807 {
		panic("Unexpected i64 value: " + fmt.Sprint(i64))
	}

	if math.Abs(float64(f32-314_567.5)) > 0.0001 {
		panic("Unexpected f32 value: " + fmt.Sprint(f32))
	}

	if math.Abs(f64-9_007_199_254_740.125) > 0.0001 {
		panic("Unexpected f64 value: " + fmt.Sprint(f64))
	}
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go <wasm_file>")
		os.Exit(1)
	}

	wasmFile := os.Args[1]
	data, err := os.ReadFile(wasmFile)
	if err != nil {
		fmt.Printf("Failed to read wasm file: %v\n", err)
		os.Exit(1)
	}

	manifest := extism.Manifest{
		Wasm: []extism.Wasm{extism.WasmData{Data: data}},
	}

	extism.SetLogLevel(extism.LogLevelDebug)

	ctx := context.Background()
	config := extism.PluginConfig{EnableWasi: true}
	plugin, err := extism.NewPlugin(ctx, manifest, config, []extism.HostFunction{
		extism.NewHostFunctionWithStack("capitalize", capitalize, []extism.ValueType{extism.ValueTypePTR}, []extism.ValueType{extism.ValueTypePTR}),
		extism.NewHostFunctionWithStack("floatInputs", floatInputs, []extism.ValueType{extism.ValueTypeF64, extism.ValueTypeF32}, []extism.ValueType{extism.ValueTypeI32}),
		extism.NewHostFunctionWithStack("floatOutput", floatOutput, []extism.ValueType{extism.ValueTypeI32}, []extism.ValueType{extism.ValueTypeF64}),
		extism.NewHostFunctionWithStack("voidInputs", voidInputs, []extism.ValueType{extism.ValueTypeI32, extism.ValueTypeI64, extism.ValueTypeF32, extism.ValueTypeF64, extism.ValueTypeI32}, []extism.ValueType{}),
	})

	if err != nil {
		fmt.Printf("Failed to initialize plugin: %v\n", err)
		os.Exit(1)
	}

	exit, result, err := plugin.Call("greet", []byte("Benjamin"))
	if err != nil {
		fmt.Printf("Plugin call failed: %v\n", err)
		os.Exit(int(exit))
	}

	fmt.Println(string(result))
}
