import React from "react";
import {Composition} from "remotion";

import {AgentExporterPromo} from "./Promo";
import {AgentExporterSocialCard} from "./SocialCard";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="AgentExporterPromo"
        component={AgentExporterPromo}
        durationInFrames={540}
        fps={30}
        width={1280}
        height={720}
        defaultProps={{}}
      />
      <Composition
        id="AgentExporterSocialCard"
        component={AgentExporterSocialCard}
        durationInFrames={1}
        fps={30}
        width={1200}
        height={630}
        defaultProps={{}}
      />
    </>
  );
};
