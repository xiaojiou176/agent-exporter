import React from "react";
import {Composition} from "remotion";

import {AgentExporterPromo} from "./Promo";

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
    </>
  );
};
