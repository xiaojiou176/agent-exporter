import React from "react";
import {
  AbsoluteFill,
  Audio,
  Img,
  interpolate,
  Sequence,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

const palette = {
  canvas: "#F8FBFF",
  depth: "#E8EEF8",
  surface: "rgba(255,255,255,0.9)",
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

const shellCard: React.CSSProperties = {
  background: palette.surface,
  border: `1px solid ${palette.line}`,
  borderRadius: 28,
  boxShadow:
    "0 28px 60px rgba(15,23,42,0.08), 0 0 0 1px rgba(255,255,255,0.8) inset",
  backdropFilter: "blur(16px)",
};

const appear = (frame: number, start: number, duration: number) =>
  interpolate(frame, [start, start + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

const Card: React.FC<{
  title: string;
  body: string;
  label: string;
  frame: number;
  start: number;
  width?: number | string;
}> = ({title, body, label, frame, start, width = 360}) => {
  const progress = spring({
    frame: frame - start,
    fps: 30,
    config: {
      damping: 18,
      stiffness: 110,
      mass: 0.9,
    },
  });

  return (
    <div
      style={{
        ...shellCard,
        width,
        padding: 24,
        transform: `translateY(${interpolate(progress, [0, 1], [24, 0])}px) scale(${interpolate(
          progress,
          [0, 1],
          [0.98, 1],
        )})`,
        opacity: progress,
      }}
    >
      <div
        style={{
          fontFamily: monoStack,
          fontSize: 14,
          letterSpacing: "0.14em",
          textTransform: "uppercase",
          color: palette.accent,
          marginBottom: 14,
        }}
      >
        {label}
      </div>
      <div
        style={{
          fontSize: 34,
          lineHeight: 1.05,
          fontWeight: 650,
          color: palette.ink,
          marginBottom: 14,
        }}
      >
        {title}
      </div>
      <div
        style={{
          fontSize: 22,
          lineHeight: 1.45,
          color: palette.inkSoft,
        }}
      >
        {body}
      </div>
    </div>
  );
};

const LanePill: React.FC<{
  title: string;
  detail: string;
  active?: boolean;
}> = ({title, detail, active = false}) => {
  return (
    <div
      style={{
        ...shellCard,
        flex: 1,
        padding: 20,
        background: active ? "rgba(255,255,255,0.96)" : "rgba(255,255,255,0.84)",
      }}
    >
      <div
        style={{
          fontFamily: monoStack,
          fontSize: 13,
          letterSpacing: "0.12em",
          textTransform: "uppercase",
          color: active ? palette.accentStrong : palette.muted,
          marginBottom: 10,
        }}
      >
        {title}
      </div>
      <div
        style={{
          fontSize: 18,
          lineHeight: 1.45,
          color: palette.inkSoft,
        }}
      >
        {detail}
      </div>
    </div>
  );
};

export const AgentExporterPromo: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();

  const heroOpacity = appear(frame, 0, 24);
  const subtleDrift = interpolate(frame, [0, 540], [0, -24], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        fontFamily: fontStack,
        background: `radial-gradient(circle at 16% 18%, rgba(37,99,235,0.12), transparent 0 28%), radial-gradient(circle at 86% 10%, rgba(14,165,233,0.10), transparent 0 24%), linear-gradient(180deg, ${palette.canvas} 0%, ${palette.depth} 100%)`,
        color: palette.ink,
        overflow: "hidden",
      }}
    >
      <Audio src={staticFile("agent-exporter-promo-landscape-voiceover.m4a")} volume={0.92} />
      <AbsoluteFill
        style={{
          opacity: 0.25,
          backgroundImage:
            "linear-gradient(rgba(148,163,184,0.08) 1px, transparent 1px), linear-gradient(90deg, rgba(148,163,184,0.08) 1px, transparent 1px)",
          backgroundSize: "48px 48px",
          transform: `translateY(${subtleDrift}px)`,
        }}
      />

      <Sequence from={0} durationInFrames={150}>
        <AbsoluteFill
          style={{
            padding: "72px 88px",
            justifyContent: "space-between",
          }}
        >
          <div
            style={{
              ...shellCard,
              padding: 34,
              display: "grid",
              gridTemplateColumns: "1.5fr 0.9fr",
              gap: 32,
              opacity: heroOpacity,
              transform: `translateY(${interpolate(heroOpacity, [0, 1], [18, 0])}px)`,
            }}
          >
            <div style={{display: "flex", flexDirection: "column", gap: 18}}>
              <div
                style={{
                  fontFamily: monoStack,
                  fontSize: 15,
                  letterSpacing: "0.16em",
                  textTransform: "uppercase",
                  color: palette.accent,
                }}
              >
                transcript-first workbench
              </div>
              <div
                style={{
                  fontSize: 66,
                  lineHeight: 0.96,
                  fontWeight: 650,
                  letterSpacing: "-0.05em",
                }}
              >
                Export one transcript. Build one local workbench.
              </div>
              <div
                style={{
                  fontSize: 28,
                  lineHeight: 1.45,
                  color: palette.inkSoft,
                  maxWidth: 720,
                }}
              >
                Run scaffold and connectors, export one HTML receipt, then
                publish the archive shell that keeps reports plus governance
                evidence on one inspectable desk.
              </div>
              <div style={{display: "flex", gap: 14}}>
                <div
                  style={{
                    borderRadius: 999,
                    background: palette.accent,
                    color: "white",
                    padding: "12px 18px",
                    fontSize: 18,
                    fontWeight: 600,
                  }}
                >
                  HTML receipt
                </div>
                <div
                  style={{
                    borderRadius: 999,
                    background: palette.accentSoft,
                    color: palette.accentStrong,
                    padding: "12px 18px",
                    fontSize: 18,
                    fontWeight: 600,
                  }}
                >
                  Archive shell proof
                </div>
              </div>
            </div>

            <div
              style={{
                ...shellCard,
                padding: 28,
                background:
                  "linear-gradient(180deg, rgba(255,255,255,0.98), rgba(248,250,252,0.84))",
                display: "flex",
                flexDirection: "column",
                justifyContent: "space-between",
              }}
            >
              <div
                style={{
                  fontFamily: monoStack,
                  fontSize: 14,
                  letterSpacing: "0.14em",
                  textTransform: "uppercase",
                  color: palette.accent,
                }}
              >
                what you get
              </div>
              <div style={{display: "grid", gap: 14}}>
                {[
                  "one local HTML transcript receipt",
                  "one archive shell entrypoint",
                  "reports and governance as companion lanes",
                ].map((line) => (
                  <div
                    key={line}
                    style={{
                      padding: "14px 16px",
                      borderRadius: 18,
                      background: "rgba(255,255,255,0.76)",
                      border: `1px solid ${palette.line}`,
                      fontSize: 19,
                      color: palette.inkSoft,
                    }}
                  >
                    {line}
                  </div>
                ))}
              </div>
              <div
                style={{
                  fontSize: 18,
                  lineHeight: 1.5,
                  color: palette.muted,
                }}
              >
                local-first, proof-first, and not a hosted platform.
              </div>
            </div>
          </div>

          <div style={{display: "flex", justifyContent: "center"}}>
            <Img
              src={staticFile("archive-shell-proof.svg")}
              style={{
                width: 760,
                opacity: interpolate(frame, [52, 120], [0, 1], {
                  extrapolateLeft: "clamp",
                  extrapolateRight: "clamp",
                }),
                transform: `translateY(${interpolate(frame, [52, 120], [20, 0], {
                  extrapolateLeft: "clamp",
                  extrapolateRight: "clamp",
                })}px)`,
              }}
            />
          </div>
        </AbsoluteFill>
      </Sequence>

      <Sequence from={150} durationInFrames={110}>
        <AbsoluteFill style={{padding: "76px 88px", justifyContent: "center"}}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 24,
            }}
          >
            <div
              style={{
                fontFamily: monoStack,
                fontSize: 15,
                letterSpacing: "0.16em",
                textTransform: "uppercase",
                color: palette.accent,
                marginBottom: 8,
              }}
            >
              first success path
            </div>
            <div
              style={{
                fontSize: 56,
                lineHeight: 1.02,
                fontWeight: 650,
                letterSpacing: "-0.04em",
                maxWidth: 900,
              }}
            >
              Three steps to a real local receipt.
            </div>
            <div style={{display: "flex", gap: 18}}>
              <Card
                label="01"
                title="Read the bench shape"
                body="Run scaffold, then connectors, before you point the repo at a real thread."
                frame={frame}
                start={156}
              />
              <Card
                label="02"
                title="Export one transcript"
                body="Create a browsable HTML receipt inside .agents/Conversations/."
                frame={frame}
                start={168}
              />
              <Card
                label="03"
                title="Publish the archive shell"
                body="Organize transcript, reports, and evidence into one local navigation surface."
                frame={frame}
                start={180}
              />
            </div>
          </div>
        </AbsoluteFill>
      </Sequence>

      <Sequence from={260} durationInFrames={100}>
        <AbsoluteFill style={{padding: "82px 88px", justifyContent: "center"}}>
          <div style={{display: "grid", gridTemplateColumns: "1fr 1.1fr", gap: 28}}>
            <div
              style={{
                ...shellCard,
                padding: 28,
                display: "flex",
                flexDirection: "column",
                gap: 18,
              }}
            >
              <div
                style={{
                  fontFamily: monoStack,
                  fontSize: 15,
                  letterSpacing: "0.16em",
                  textTransform: "uppercase",
                  color: palette.accent,
                }}
              >
                proof ladder
              </div>
              <div
                style={{
                  fontSize: 48,
                  lineHeight: 1.02,
                  fontWeight: 650,
                  letterSpacing: "-0.04em",
                }}
              >
                Confidence climbs in order.
              </div>
              <div
                style={{
                  fontSize: 24,
                  lineHeight: 1.5,
                  color: palette.inkSoft,
                }}
              >
                CLI proof first. Transcript receipt second. Archive shell proof
                third. Do not turn proof into platform theatre.
              </div>
            </div>
            <div
              style={{
                ...shellCard,
                padding: 24,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Img
                src={staticFile("proof-ladder.svg")}
                style={{
                  width: "100%",
                  opacity: appear(frame, 266, 20),
                  transform: `scale(${interpolate(frame, [266, 320], [0.96, 1], {
                    extrapolateLeft: "clamp",
                    extrapolateRight: "clamp",
                  })})`,
                }}
              />
            </div>
          </div>
        </AbsoluteFill>
      </Sequence>

      <Sequence from={360} durationInFrames={96}>
        <AbsoluteFill style={{padding: "86px 88px", justifyContent: "center"}}>
          <div style={{display: "grid", gap: 22}}>
            <div
              style={{
                fontFamily: monoStack,
                fontSize: 15,
                letterSpacing: "0.16em",
                textTransform: "uppercase",
                color: palette.accent,
              }}
            >
              lane hierarchy
            </div>
            <div
              style={{
                fontSize: 56,
                lineHeight: 1.02,
                fontWeight: 650,
                letterSpacing: "-0.04em",
                maxWidth: 940,
              }}
            >
              Transcript is the main route. Search and governance are lanes.
            </div>
            <div style={{display: "flex", gap: 18}}>
              <LanePill
                title="Main route"
                detail="archive / transcript workbench"
                active
              />
              <LanePill
                title="Side lane"
                detail="search / retrieval reports"
              />
              <LanePill
                title="Side lane"
                detail="integration evidence / governance"
              />
            </div>
            <div
              style={{
                ...shellCard,
                padding: 22,
                display: "flex",
                gap: 16,
                alignItems: "center",
              }}
            >
              <div
                style={{
                  background: palette.deep,
                  color: "white",
                  borderRadius: 20,
                  padding: "14px 16px",
                  fontFamily: monoStack,
                  fontSize: 16,
                  minWidth: 250,
                }}
              >
                evidence before theatre
              </div>
              <div
                style={{
                  fontSize: 21,
                  color: palette.inkSoft,
                  lineHeight: 1.5,
                }}
              >
                The workbench can route transcripts, reports, and governance
                without pretending to be a hosted platform.
              </div>
            </div>
          </div>
        </AbsoluteFill>
      </Sequence>

      <Sequence from={456} durationInFrames={84}>
        <AbsoluteFill style={{padding: "82px 88px", justifyContent: "center"}}>
          <div
            style={{
              ...shellCard,
              padding: 40,
              display: "grid",
              gridTemplateColumns: "1.2fr 0.8fr",
              gap: 28,
              alignItems: "center",
            }}
          >
            <div style={{display: "grid", gap: 16}}>
              <div
                style={{
                  fontFamily: monoStack,
                  fontSize: 15,
                  letterSpacing: "0.16em",
                  textTransform: "uppercase",
                  color: palette.accent,
                }}
              >
                what to do next
              </div>
              <div
                style={{
                  fontSize: 58,
                  lineHeight: 0.98,
                  fontWeight: 650,
                  letterSpacing: "-0.04em",
                }}
              >
                Try it in 3 steps. Then inspect the proof lane.
              </div>
              <div
                style={{
                  fontSize: 24,
                  lineHeight: 1.5,
                  color: palette.inkSoft,
                  maxWidth: 720,
                }}
              >
                README, Pages docs, and the proof lane now tell the same story:
                export first, inspect the proof, then open companion lanes.
              </div>
            </div>
            <div
              style={{
                display: "grid",
                gap: 14,
              }}
            >
              {[
                "README / GitHub front door",
                "Pages docs landing",
                "Archive shell proof",
                "Published shelf / release notes",
              ].map((line, index) => (
                <div
                  key={line}
                  style={{
                    ...shellCard,
                    padding: "16px 18px",
                    opacity: appear(frame, 462 + index * 6, 14),
                    transform: `translateY(${interpolate(
                      frame,
                      [462 + index * 6, 486 + index * 6],
                      [12, 0],
                      {
                        extrapolateLeft: "clamp",
                        extrapolateRight: "clamp",
                      },
                    )}px)`,
                    fontSize: 20,
                    color: palette.inkSoft,
                  }}
                >
                  {line}
                </div>
              ))}
            </div>
          </div>
        </AbsoluteFill>
      </Sequence>
    </AbsoluteFill>
  );
};
