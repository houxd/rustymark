# Rustymark

Rustmark is a little gadget for adding watermarks to images, written in Rust.

### Notice: WIP

This project is still a work in progress, is ready for use but not user-friendly yet.

### Usage

```bash
cli under construction
```

Just dry run once, it will generate a `config.toml` file in the current directory, you can modify it to your needs. Then run it again to generate the watermarked images.


### Configurations

| Key    | Description                                                      | Default       |
|--------|------------------------------------------------------------------|---------------|
| `text` | The input directory                                              | Empty         |
| `font_path` | Font file loaded by rusttype                                     | `./msyh.ttc`  |
| `image_path` | Input file name                                                  | `./tests.png` |
| `angle` | Angle of the text, used for dividing Π                           | `-6.0`         |
| `color` | Color of the text                                                | `[0, 0, 0, 100]` |
|`margin` | Margin between watermark's edge and **single** picture's boarder | `10`          |
| `alpha` | Background transparency detection argument                       | `0`           |


