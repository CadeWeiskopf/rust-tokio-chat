import React, { useContext } from "react";
import { AppContext } from "../../App.context";

export const Game: React.FC = () => {
  const { currentMatch } = useContext(AppContext);
  if (!currentMatch) {
    throw Error(" no current match ");
  }
  return (
    <div>
      <h2>in game</h2>
      <h3>{currentMatch.users.map((e) => e.name).join(", ")}</h3>
    </div>
  );
};
