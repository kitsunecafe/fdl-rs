# FDL-rs
[File Description Language](https://github.com/razziefox/FDL) parser for Rust

# What's FDL?
FDL is a dead simple file format specifically designed to store metadata about assets such as sprites. Made by the talented [razziefox](https://github.com/razziefox)

# Example
```rs
-- main.rs

fn main() {
    if let Ok(fdl) = FDL::load_from_file("test.fdl") {
        if let Some(flap_frames) = fdl.fetch("flap", "frames") {
            assert_eq!(flap_frames, "1");
        }
    }
}
```

```
-- player.fdl

[sprite]
file=player.png
frames=6
width=8
height=8
[/]

[walk]
frames=3
begin=1
end=3
[/]

[flap]
frames=1
begin=4
end=4
[/]

[fall]
frames=1
begin=5
end=5
[/]

[hurt]
frames=1
begin=6
end=6
[/]

```
