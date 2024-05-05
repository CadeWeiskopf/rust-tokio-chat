import React, { useEffect, useRef } from "react";

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
  }, []);

  return (
    <canvas
      ref={canvasRef}
      width={400}
      height={400}
    />
  );
};
