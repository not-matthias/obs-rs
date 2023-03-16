# obs-client-ffi

## Requirements

```
cargo +nightly install cbindgen
```

## Headers 

Generate with:
```
cbindgen --config cbindgen.toml --crate obs-client-ffi --output obs-client.h -l C    
cbindgen --config cbindgen.toml --crate obs-client-ffi --output obs-client.hpp -l C++   
```