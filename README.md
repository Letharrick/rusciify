<img src="./meta/logo.png" width="292" height="76">

A CLI tool for converting images to ASCII

# Usage
```bash
git clone https://github.com/Letharrick/rusciify.git
cd rusciify
cargo run --release -- --help
```

# Examples
<img src="./examples/image.jpg" width="250" height="250">

[Photo source](https://unsplash.com/photos/lylCw4zcA7I)
<br/>

### Basic
```
rusciify example.jpg -o ascii
```
<img src="./examples/ascii.png" width="250" height="250">

### Custom character map
```
rusciify example.jpg -o ascii_nums -c 0123456789
```
<img src="./examples/ascii_nums.png" width="250" height="250">

### Solid character map
```
rusciify example.jpg -o ascii_solid -s
```
<img src="./examples/ascii_solid.png" width="250" height="250">

### Custom character scale
```
rusciify example.jpg -o ascii_solid_scaled -s -a 25
```
<img src="./examples/ascii_solid_scaled.png" width="250" height="250">