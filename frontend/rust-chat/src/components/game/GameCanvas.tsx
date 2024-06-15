import React, { useContext, useEffect, useRef, useState } from "react";
import { AppContext, GamePiece } from "../../App.context";
import { SHAPES } from "./game.model";

export const GameCanvas: React.FC = () => {
  const { gamePieces } = useContext(AppContext);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [canvasSize, setCanvasSize] = useState({ width: 400, height: 400 });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      throw Error("no canvas ref");
    }
    const context = canvas.getContext("2d");
    if (!context) {
      throw Error("no context");
    }
    const gridWidth = 20;
    const gridHeight = 24;
    const cellSize = Math.min(
      canvasSize.width / gridWidth,
      canvasSize.height / gridHeight
    );
    const render = requestAnimationFrame(() => {
      context.clearRect(0, 0, canvas.width, canvas.height);
      gamePieces.forEach(({ shape, position }) => {
        context.fillStyle = "black";
        const shapeCoords = SHAPES[shape];
        shapeCoords.forEach(([dx, dy]) => {
          context.fillRect(
            position.x * cellSize + dx * cellSize,
            position.y * cellSize + dy * cellSize,
            cellSize,
            cellSize
          );
        });
      });
    });

    return () => {
      cancelAnimationFrame(render);
    };
  }, [gamePieces]);

  return (
    <>
      {gamePieces.map((g) => {
        return <>{g.shape}</>;
      })}
      <canvas
        ref={canvasRef}
        style={{ border: "1px solid" }}
        width={canvasSize.width}
        height={canvasSize.height}
      />
    </>
  );
};
