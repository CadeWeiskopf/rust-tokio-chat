import React, { useContext, useEffect, useRef } from "react";
import { AppContext, GamePiece } from "../../App.context";
import { SHAPES } from "./game.model";

export const GameCanvas: React.FC = () => {
  const { gamePieces } = useContext(AppContext);
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
    const size = 20;
    gamePieces.forEach(({ shape, position }) => {
      context.fillStyle = "black";
      const shapeCoords = SHAPES[shape];
      shapeCoords.forEach(([dx, dy]) => {
        context.fillRect(
          position.x + dx * size,
          position.y + dy * size,
          size,
          size
        );
      });
    });
  }, [gamePieces]);

  return (
    <>
      {gamePieces.map((g) => {
        return <>{g.shape}</>;
      })}
      <canvas
        ref={canvasRef}
        width={400}
        height={400}
      />
    </>
  );
};
