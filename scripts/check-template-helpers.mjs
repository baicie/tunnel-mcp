import { mkdirSync } from "node:fs";

const action = process.argv[2];

switch (action) {
  case "ensure-dist":
    mkdirSync("dist", { recursive: true });
    console.log("dist placeholder ready.");
    break;
  default:
    console.error(`Unknown helper action: ${action}`);
    process.exitCode = 1;
}
