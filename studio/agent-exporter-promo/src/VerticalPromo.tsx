import React from "react";
import {
  AbsoluteFill,
  Audio,
  Img,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
} from "remotion";

const palette = {
  canvas: "#F8FBFF",
  depth: "#E8EEF8",
  surface: "rgba(255,255,255,0.92)",
  line: "rgba(15,23,42,0.08)",
  ink: "#0F172A",
  inkSoft: "#334155",
  muted: "#64748B",
  accent: "#2563EB",
  accentStrong: "#1D4ED8",
  accentSoft: "rgba(37,99,235,0.10)",
  deep: "#0F172A",
};

const fontStack =
  "\"IBM Plex Sans\", \"SF Pro Display\", \"Segoe UI\", system-ui, sans-serif";
const monoStack =
  "\"JetBrains Mono\", \"SFMono-Regular\", \"Menlo\", monospace";

const reveal = (frame: number, start: number) =>
  spring({
    frame: frame - start,
    fps: 30,
    config: {
      damping: 18,
      stiffness: 110,
      mass: 0.9,
    },
  });

const panel: React.CSSProperties = {
  background: palette.surface,
  border: `1px solid ${palette.line}`,
  borderRadius: 34,
  boxShadow:
    "0 30px 70px rgba(15,23,42,0.08), 0 0 0 1px rgba(255,255,255,0.82) inset",
};

export const AgentExporterVerticalPromo: React.FC = () => {
  const frame = useCurrentFrame();

  const hero = reveal(frame, 0);
  const middle = reveal(frame, 120);
  const proof = reveal(frame, 260);

  return (
    <AbsoluteFill
      style={{
        fontFamily: fontStack,
        background: `radial-gradient(circle at 18% 14%, rgba(37,99,235,0.16), transparent 0 26%), radial-gradient(circle at 82% 8%, rgba(14,165,233,0.10), transparent 0 24%), linear-gradient(180deg, ${palette.canvas} 0%, ${palette.depth} 100%)`,
        color: palette.ink,
      }}
    >
      <Audio src={staticFile("agent-exporter-promo-vertical-voiceover.m4a")} volume={0.94} />
      <AbsoluteFill
        style={{
          opacity: 0.2,
          backgroundImage:
            "linear-gradient(rgba(148,163,184,0.08) 1px, transparent 1px), linear-gradient(90deg, rgba(148,163,184,0.08) 1px, transparent 1px)",
          backgroundSize: "54px 54px",
        }}
      />

      <div
        style={{
          position: "absolute",
          inset: 58,
          display: "grid",
          gridTemplateRows: "auto auto auto 1fr auto",
          gap: 24,
        }}
      >
        <div
          style={{
            ...panel,
            padding: 34,
            opacity: hero,
            transform: `translateY(${interpolate(hero, [0, 1], [24, 0])}px)`,
          }}
        >
          <div
            style={{
              fontFamily: monoStack,
              fontSize: 22,
              letterSpacing: "0.16em",
              textTransform: "uppercase",
              color: palette.accent,
              marginBottom: 20,
            }}
          >
            transcript-first workbench
          </div>
          <div
            style={{
              fontSize: 92,
              lineHeight: 0.94,
              fontWeight: 650,
              letterSpacing: "-0.06em",
              marginBottom: 20,
            }}
          >
            Export one transcript. Build one local workbench.
          </div>
          <div
            style={{
              fontSize: 42,
              lineHeight: 1.34,
              color: palette.inkSoft,
            }}
          >
            HTML receipt first. Archive proof second. Companion lanes after that.
          </div>
        </div>

        <div
          style={{
            ...panel,
            padding: 28,
            opacity: middle,
            transform: `translateY(${interpolate(middle, [0, 1], [24, 0])}px)`,
          }}
        >
          <div
            style={{
              fontFamily: monoStack,
              fontSize: 20,
              letterSpacing: "0.14em",
              textTransform: "uppercase",
              color: palette.accent,
              marginBottom: 16,
            }}
          >
            first success
          </div>
          <ol
            style={{
              margin: 0,
              paddingLeft: 28,
              display: "grid",
              gap: 12,
              fontSize: 34,
              lineHeight: 1.35,
              color: palette.inkSoft,
            }}
          >
            <li>run `scaffold` and `connectors`</li>
            <li>export one HTML transcript receipt</li>
            <li>publish the archive shell proof</li>
          </ol>
        </div>

        <div
          style={{
            ...panel,
            padding: 28,
            opacity: proof,
            transform: `translateY(${interpolate(proof, [0, 1], [24, 0])}px)`,
          }}
        >
          <div
            style={{
              fontFamily: monoStack,
              fontSize: 20,
              letterSpacing: "0.14em",
              textTransform: "uppercase",
              color: palette.accent,
              marginBottom: 14,
            }}
          >
            proof ladder
          </div>
          <div
            style={{
              fontSize: 38,
              lineHeight: 1.3,
              color: palette.inkSoft,
              marginBottom: 18,
            }}
          >
            CLI quickstart first. Archive shell proof second. Secondary lanes
            after that.
          </div>
          <div
            style={{
              borderRadius: 24,
              background: palette.deep,
              color: "white",
              padding: "18px 20px",
              fontFamily: monoStack,
              fontSize: 24,
              lineHeight: 1.35,
            }}
          >
            not a hosted platform
            <br />
            not a generic MCP product
          </div>
        </div>

        <div
          style={{
            ...panel,
            padding: 24,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            overflow: "hidden",
          }}
        >
          <Img
            src={staticFile("proof-ladder.svg")}
            style={{
              width: "92%",
              opacity: interpolate(frame, [280, 360], [0.3, 1], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            }}
          />
        </div>

        <div
          style={{
            display: "flex",
            gap: 14,
          }}
        >
          <div
            style={{
              flex: 1,
              borderRadius: 999,
              background: palette.accent,
              color: "white",
              padding: "18px 20px",
              textAlign: "center",
              fontSize: 26,
              fontWeight: 600,
            }}
          >
            Watch the promo reel
          </div>
          <div
            style={{
              flex: 1,
              borderRadius: 999,
              background: palette.accentSoft,
              color: palette.accentStrong,
              padding: "18px 20px",
              textAlign: "center",
              fontSize: 26,
              fontWeight: 600,
            }}
          >
            Run the quickstart path
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
