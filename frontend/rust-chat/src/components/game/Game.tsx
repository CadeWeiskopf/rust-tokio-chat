import React, { useEffect, useRef } from "react";
import { drawUser } from "./assets/user";
import testMap from "./test-map";

export const Game: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      throw Error("no canvas ref");
    }
    const context = canvas.getContext("2d");
    if (!context) {
      throw Error("no context");
    }

    drawUser(context, { x: 200, y: 200 });
    console.log(testMap);
    for (let i = 0; i < testMap.length; i++) {
      for (let j = 0; j < testMap[i].length; j++) {
        // If the value is '1', draw a black square
        if (testMap[i][j] === "1") {
          context.fillStyle = "black";
          context.fillRect(j, i, 1, 1);
        }
      }
    }
  }, []);

  return (
    <canvas
      ref={canvasRef}
      width={400}
      height={400}
    />
  );
};
