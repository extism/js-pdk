(module
  (func $invoke (export "__invoke") (param $funcid i32) (result i32)
      local.get $funcid
      return
  )
  (func $greet (export "greet") (result i32)
        i32.const 1
        call $invoke
  )
)
