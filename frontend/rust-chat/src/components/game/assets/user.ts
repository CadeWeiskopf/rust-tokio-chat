export type Position = { x: number; y: number };
const HEAD_SCALE = 8;
const BODY_SCALE = 35;
const ARM_SCALE = 15;
const LEG_SCALE = 25;

export function drawUser(
  context: CanvasRenderingContext2D,
  position: Position
) {
  // Draw head
  context.beginPath();
  context.arc(200, 50, HEAD_SCALE, 0, Math.PI * 2);
  context.fillStyle = "black";
  context.fill();
  context.closePath();

  // Draw body
  context.beginPath();
  context.moveTo(200, 50);
  context.lineTo(200, 50 + BODY_SCALE);
  context.strokeStyle = "black";
  context.stroke();
  context.closePath();

  // Draw arms
  const armPos = { x: 200, y: 50 + BODY_SCALE / 2 };
  context.beginPath();
  context.moveTo(armPos.x, armPos.y);
  context.lineTo(armPos.x - ARM_SCALE, armPos.y + ARM_SCALE);
  context.moveTo(armPos.x, armPos.y);
  context.lineTo(armPos.x + ARM_SCALE, armPos.y + ARM_SCALE);
  context.strokeStyle = "black";
  context.stroke();
  context.closePath();

  // Draw legs
  const legPos = { x: 200, y: 50 + BODY_SCALE };
  context.beginPath();
  context.moveTo(legPos.x, legPos.y);
  context.lineTo(legPos.x - LEG_SCALE / 2, legPos.y + LEG_SCALE);
  context.moveTo(legPos.x, legPos.y);
  context.lineTo(legPos.x + LEG_SCALE / 2, legPos.y + LEG_SCALE);

  context.strokeStyle = "black";
  context.stroke();
  context.closePath();
}
