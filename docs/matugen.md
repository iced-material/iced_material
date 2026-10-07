# Color scheme files

A color scheme can be loaded from the JSON that Matugen prints, so that a wallpaper-derived palette drives the whole interface. The format is read by `material_iced::color::matugen::import` and written by `matugen::export`.

## Layout

```json
{
  "colors": {
    "primary": {
      "light": { "color": "#6750a4" },
      "dark": { "color": "#cfbcff" }
    },
    "on_primary": {
      "light": { "color": "#ffffff" },
      "dark": { "color": "#381e72" }
    }
  }
}
```

- The top level has one object, `colors`.
- `colors` has one key for each color role, with the snake case name of the role, for example `on_primary_container` or `surface_container_high`. `Role::ALL` lists all 49 roles and `Role::name` gives their names.
- Each role has the keys `light` and `dark`. Each of them is an object with the key `color`, or directly the color string. A key `default` is written by `export` and ignored by `import`.
- A color is `#RRGGBB` or `#RRGGBBAA`, in hexadecimal.
- Every role has to be present for both modes. A missing role or an invalid color is an error that names the role and the mode.
- Other keys are ignored.

The layout is the one Matugen 4 prints with `--json hex`.

## Derived roles

The scheme of this library has two roles that Matugen does not have, `success` and `warning`. They are not read from the file. They are derived from the imported `primary` in the same way as for a generated scheme: the green and amber seeds are harmonized toward `primary`.

## Use

```rust
use material_iced::color::matugen;

let json = std::fs::read_to_string("colors.json")?;
let (light, dark) = matugen::import(&json)?;
let theme = material_iced::Theme::with_colors(if want_dark { dark } else { light }, want_dark);
```

`matugen::export(&light, &dark, dark_is_default)` writes the two schemes in the same layout. The gallery loads a file from the path typed in the side panel and switches between the loaded schemes with the same animated transition as between generated ones.
