// SPDX-License-Identifier: LGPL-3.0-only

import blend.Blend;
import contrast.Contrast;
import dynamiccolor.ColorSpec.SpecVersion;
import dynamiccolor.DynamicColor;
import dynamiccolor.DynamicScheme;
import dynamiccolor.MaterialDynamicColors;
import hct.Hct;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import palettes.CorePalette;
import palettes.TonalPalette;
import quantize.QuantizerCelebi;
import scheme.SchemeContent;
import scheme.SchemeExpressive;
import scheme.SchemeFidelity;
import scheme.SchemeFruitSalad;
import scheme.SchemeMonochrome;
import scheme.SchemeNeutral;
import scheme.SchemeRainbow;
import scheme.SchemeTonalSpot;
import scheme.SchemeVibrant;
import score.Score;

public class Gen {
  static long state = 0x2545F491L;

  static int next() {
    state = (state * 6364136223846793005L + 1442695040888963407L);
    return (int) (state >>> 33);
  }

  static String hex(int argb) {
    return String.format("%06x", argb & 0xFFFFFF);
  }

  static DynamicScheme scheme(String variant, Hct source, boolean dark, double contrast, SpecVersion spec) {
    DynamicScheme.Platform phone = DynamicScheme.Platform.PHONE;
    return switch (variant) {
      case "monochrome" -> new SchemeMonochrome(source, dark, contrast, spec, phone);
      case "neutral" -> new SchemeNeutral(source, dark, contrast, spec, phone);
      case "tonal_spot" -> new SchemeTonalSpot(source, dark, contrast, spec, phone);
      case "vibrant" -> new SchemeVibrant(source, dark, contrast, spec, phone);
      case "expressive" -> new SchemeExpressive(source, dark, contrast, spec, phone);
      case "fidelity" -> new SchemeFidelity(source, dark, contrast, spec, phone);
      case "content" -> new SchemeContent(source, dark, contrast, spec, phone);
      case "rainbow" -> new SchemeRainbow(source, dark, contrast, spec, phone);
      default -> new SchemeFruitSalad(source, dark, contrast, spec, phone);
    };
  }

  public static void main(String[] args) throws Exception {
    Path out = Path.of(args[0]);
    MaterialDynamicColors c = new MaterialDynamicColors();
    List<DynamicColor> roles = List.of(
        c.primary(),
        c.onPrimary(),
        c.primaryContainer(),
        c.onPrimaryContainer(),
        c.secondary(),
        c.onSecondary(),
        c.secondaryContainer(),
        c.onSecondaryContainer(),
        c.tertiary(),
        c.onTertiary(),
        c.tertiaryContainer(),
        c.onTertiaryContainer(),
        c.error(),
        c.onError(),
        c.errorContainer(),
        c.onErrorContainer(),
        c.background(),
        c.onBackground(),
        c.surface(),
        c.onSurface(),
        c.surfaceVariant(),
        c.onSurfaceVariant(),
        c.surfaceDim(),
        c.surfaceBright(),
        c.surfaceContainerLowest(),
        c.surfaceContainerLow(),
        c.surfaceContainer(),
        c.surfaceContainerHigh(),
        c.surfaceContainerHighest(),
        c.surfaceTint(),
        c.outline(),
        c.outlineVariant(),
        c.shadow(),
        c.scrim(),
        c.inverseSurface(),
        c.inverseOnSurface(),
        c.inversePrimary(),
        c.primaryFixed(),
        c.primaryFixedDim(),
        c.onPrimaryFixed(),
        c.onPrimaryFixedVariant(),
        c.secondaryFixed(),
        c.secondaryFixedDim(),
        c.onSecondaryFixed(),
        c.onSecondaryFixedVariant(),
        c.tertiaryFixed(),
        c.tertiaryFixedDim(),
        c.onTertiaryFixed(),
        c.onTertiaryFixedVariant());
    int[] seeds = {0x6750A4, 0xB3261E, 0x0B57D0, 0x00FF00, 0x0000FF, 0x000000, 0xFFFFFF, 0x808080, 0xFFDE3F,
        0x00796B, 0xE91E63, 0x8D6E63, next() & 0xFFFFFF, next() & 0xFFFFFF, next() & 0xFFFFFF, next() & 0xFFFFFF};
    String[] variants = {"monochrome", "neutral", "tonal_spot", "vibrant", "expressive", "fidelity", "content",
        "rainbow", "fruit_salad"};
    List<String> spec2025 = List.of("neutral", "tonal_spot", "vibrant", "expressive");
    double[] contrasts = {-1.0, 0.0, 0.5, 1.0};
    StringBuilder schemes = new StringBuilder();
    for (int seed : seeds) {
      for (String variant : variants) {
        for (SpecVersion spec : new SpecVersion[] {SpecVersion.SPEC_2021, SpecVersion.SPEC_2025}) {
          if (spec == SpecVersion.SPEC_2025 && !spec2025.contains(variant)) {
            continue;
          }
          for (boolean dark : new boolean[] {false, true}) {
            for (double contrast : contrasts) {
              DynamicScheme s = scheme(variant, Hct.fromInt(0xFF000000 | seed), dark, contrast, spec);
              schemes.append(hex(seed)).append(' ').append(variant).append(' ')
                  .append(spec == SpecVersion.SPEC_2021 ? "2021" : "2025").append(' ')
                  .append(dark ? 1 : 0).append(' ').append(contrast);
              for (DynamicColor role : roles) {
                schemes.append(' ').append(hex(s.getArgb(role)));
              }
              schemes.append('\n');
            }
          }
        }
      }
    }
    Files.writeString(out.resolve("schemes.txt"), schemes.toString());

    int[] customSeeds = {0x4CAF50, 0xFFC107};
    int[] lightTones = {40, 100, 90, 10};
    int[] darkTones = {80, 20, 30, 90};
    StringBuilder custom = new StringBuilder();
    for (int seed : seeds) {
      for (String variant : variants) {
        for (SpecVersion spec : new SpecVersion[] {SpecVersion.SPEC_2021, SpecVersion.SPEC_2025}) {
          if (spec == SpecVersion.SPEC_2025 && !spec2025.contains(variant)) {
            continue;
          }
          for (boolean dark : new boolean[] {false, true}) {
            for (double contrast : new double[] {0.0, 1.0}) {
              DynamicScheme s = scheme(variant, Hct.fromInt(0xFF000000 | seed), dark, contrast, spec);
              int primary = s.getArgb(c.primary());
              custom.append(hex(seed)).append(' ').append(variant).append(' ')
                  .append(spec == SpecVersion.SPEC_2021 ? "2021" : "2025").append(' ')
                  .append(dark ? 1 : 0).append(' ').append(contrast).append(' ').append(hex(primary));
              for (int customSeed : customSeeds) {
                int harmonized = Blend.harmonize(0xFF000000 | customSeed, primary);
                TonalPalette tones = CorePalette.of(harmonized).a1;
                for (int tone : dark ? darkTones : lightTones) {
                  custom.append(' ').append(hex(tones.tone(tone)));
                }
              }
              custom.append('\n');
            }
          }
        }
      }
    }
    Files.writeString(out.resolve("custom.txt"), custom.toString());

    StringBuilder hct = new StringBuilder();
    for (int i = 0; i < 300; i++) {
      int argb = 0xFF000000 | (next() & 0xFFFFFF);
      Hct h = Hct.fromInt(argb);
      hct.append(hex(argb)).append(' ').append(h.getHue()).append(' ').append(h.getChroma()).append(' ')
          .append(h.getTone()).append('\n');
    }
    Files.writeString(out.resolve("hct.txt"), hct.toString());

    StringBuilder solver = new StringBuilder();
    for (int i = 0; i < 300; i++) {
      double hue = Math.floorMod(next(), 36000) / 100.0;
      double chroma = Math.floorMod(next(), 15000) / 100.0;
      double tone = Math.floorMod(next(), 10000) / 100.0;
      solver.append(hue).append(' ').append(chroma).append(' ').append(tone).append(' ')
          .append(hex(Hct.from(hue, chroma, tone).toInt())).append('\n');
    }
    Files.writeString(out.resolve("solver.txt"), solver.toString());

    StringBuilder palettes = new StringBuilder();
    for (int seed : seeds) {
      Hct h = Hct.fromInt(0xFF000000 | seed);
      TonalPalette p = TonalPalette.fromHueAndChroma(h.getHue(), h.getChroma());
      palettes.append(hex(seed)).append(' ').append(hex(p.getKeyColor().toInt()));
      for (int tone = 0; tone <= 100; tone += 1) {
        palettes.append(' ').append(hex(p.tone(tone)));
      }
      palettes.append('\n');
    }
    Files.writeString(out.resolve("palettes.txt"), palettes.toString());

    StringBuilder harmonize = new StringBuilder();
    for (int i = 0; i < 200; i++) {
      int design = 0xFF000000 | (next() & 0xFFFFFF);
      int source = 0xFF000000 | (next() & 0xFFFFFF);
      harmonize.append(hex(design)).append(' ').append(hex(source)).append(' ')
          .append(hex(Blend.harmonize(design, source))).append('\n');
    }
    Files.writeString(out.resolve("harmonize.txt"), harmonize.toString());

    StringBuilder contrast = new StringBuilder();
    for (int i = 0; i < 200; i++) {
      double a = Math.floorMod(next(), 10000) / 100.0;
      double b = Math.floorMod(next(), 10000) / 100.0;
      contrast.append(a).append(' ').append(b).append(' ').append(Contrast.ratioOfTones(a, b)).append(' ')
          .append(Contrast.lighter(a, 4.5)).append(' ').append(Contrast.darker(a, 4.5)).append('\n');
    }
    Files.writeString(out.resolve("contrast.txt"), contrast.toString());

    StringBuilder quantize = new StringBuilder();
    for (int k = 0; k < 8; k++) {
      int bases = 3 + k;
      int[] base = new int[bases];
      for (int i = 0; i < bases; i++) {
        base[i] = next() & 0xFFFFFF;
      }
      int[] pixels = new int[1024];
      for (int i = 0; i < pixels.length; i++) {
        int b = base[Math.floorMod(next(), bases)];
        int jitter = next();
        int r = Math.min(255, Math.max(0, ((b >> 16) & 255) + (jitter & 31) - 16));
        int g = Math.min(255, Math.max(0, ((b >> 8) & 255) + ((jitter >> 5) & 31) - 16));
        int bl = Math.min(255, Math.max(0, (b & 255) + ((jitter >> 10) & 31) - 16));
        pixels[i] = 0xFF000000 | (r << 16) | (g << 8) | bl;
      }
      Map<Integer, Integer> quantized = QuantizerCelebi.quantize(pixels, 128);
      List<Integer> ranked = Score.score(quantized);
      for (int p : pixels) {
        quantize.append(hex(p)).append(' ');
      }
      quantize.append('|');
      for (Map.Entry<Integer, Integer> e : quantized.entrySet()) {
        quantize.append(' ').append(hex(e.getKey())).append(':').append(e.getValue());
      }
      quantize.append(" |");
      for (int color : ranked) {
        quantize.append(' ').append(hex(color));
      }
      quantize.append('\n');
    }
    Files.writeString(out.resolve("quantize.txt"), quantize.toString());
  }
}
