import React from "react";
import {AbsoluteFill} from "remotion";

const palette = {
  canvas: "#F8FBFF",
  depth: "#E8EEF8",
  surface: "rgba(255,255,255,0.92)",
  line: "rgba(15,23,42,0.08)",
  ink: "#0F172A",
  inkSoft: "#334155",
  muted: "#64748B",
  accent: "#2563EB",
  accentSoft: "rgba(37,99,235,0.10)",
  deep: "#0F172A",
};

const fontStack =
  "\"IBM Plex Sans\", \"SF Pro Display\", \"Segoe UI\", system-ui, sans-serif";
const monoStack =
  "\"JetBrains Mono\", \"SFMono-Regular\", \"Menlo\", monospace";

export const AgentExporterSocialCard: React.FC = () => {
  return (
    <AbsoluteFill
      style={{
        fontFamily: fontStack,
        background: `radial-gradient(circle at 14% 18%, rgba(37,99,235,0.16), transparent 0 28%), radial-gradient(circle at 88% 10%, rgba(14,165,233,0.10), transparent 0 24%), linear-gradient(180deg, ${palette.canvas} 0%, ${palette.depth} 100%)`,
        color: palette.ink,
      }}
    >
      <AbsoluteFill
        style={{
          opacity: 0.2,
          backgroundImage:
            "linear-gradient(rgba(148,163,184,0.08) 1px, transparent 1px), linear-gradient(90deg, rgba(148,163,184,0.08) 1px, transparent 1px)",
          backgroundSize: "42px 42px",
        }}
      />

      <div
        style={{
          position: "absolute",
          inset: 44,
          borderRadius: 34,
          background: palette.surface,
          border: `1px solid ${palette.line}`,
          boxShadow:
            "0 28px 60px rgba(15,23,42,0.08), 0 0 0 1px rgba(255,255,255,0.8) inset",
          padding: "38px 40px",
          display: "grid",
          gridTemplateColumns: "1.45fr 0.85fr",
          gap: 28,
        }}
      >
        <div style={{display: "grid", gap: 16, alignContent: "start"}}>
          <div
            style={{
              fontFamily: monoStack,
              fontSize: 18,
              letterSpacing: "0.16em",
              textTransform: "uppercase",
              color: palette.accent,
            }}
          >
            archive and governance workbench
          </div>
          <div
            style={{
              fontSize: 78,
              lineHeight: 0.94,
              fontWeight: 650,
              letterSpacing: "-0.06em",
              maxWidth: 700,
            }}
          >
            Put transcript export, archive proof, and governance on one desk.
          </div>
          <div
            style={{
              fontSize: 31,
              lineHeight: 1.42,
              color: palette.inkSoft,
              maxWidth: 690,
            }}
          >
            CLI quickstart first. Archive shell proof second. Secondary lanes
            after that.
          </div>
          <div style={{display: "flex", gap: 12}}>
            <div
              style={{
                padding: "10px 16px",
                borderRadius: 999,
                background: palette.accent,
                color: "white",
                fontSize: 20,
                fontWeight: 600,
              }}
            >
              local-first
            </div>
            <div
              style={{
                padding: "10px 16px",
                borderRadius: 999,
                background: palette.accentSoft,
                color: palette.accent,
                fontSize: 20,
                fontWeight: 600,
              }}
            >
              proof before platform
            </div>
          </div>
        </div>

        <div
          style={{
            display: "grid",
            gap: 14,
            alignContent: "space-between",
          }}
        >
          <div
            style={{
              borderRadius: 28,
              border: `1px solid ${palette.line}`,
              padding: 24,
              background: "rgba(255,255,255,0.78)",
            }}
          >
            <div
              style={{
                fontFamily: monoStack,
                fontSize: 16,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
                color: palette.accent,
                marginBottom: 12,
              }}
            >
              what it is not
            </div>
            <div style={{display: "grid", gap: 10}}>
              {[
                "not a hosted archive platform",
                "not a generic MCP product",
                "not platform theatre before proof",
              ].map((line) => (
                <div
                  key={line}
                  style={{
                    borderRadius: 18,
                    padding: "12px 14px",
                    background: "rgba(255,255,255,0.92)",
                    border: `1px solid ${palette.line}`,
                    color: palette.inkSoft,
                    fontSize: 20,
                    lineHeight: 1.35,
                  }}
                >
                  {line}
                </div>
              ))}
            </div>
          </div>
          <div
            style={{
              borderRadius: 24,
              background: palette.deep,
              color: "white",
              padding: "18px 20px",
              fontFamily: monoStack,
              fontSize: 20,
              lineHeight: 1.35,
            }}
          >
            flagship packet
            <br />
            GitHub repo + CLI quickstart + archive shell proof
          </div>
          <div
            style={{
              color: palette.muted,
              fontSize: 19,
              lineHeight: 1.5,
            }}
          >
            inspectable, developer-grade, and calm enough to trust.
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
