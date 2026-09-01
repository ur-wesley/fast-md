import { spawn } from "node:child_process";

const children = [
  spawn("bun", ["run", "watch:css"], { stdio: "inherit", shell: true }),
  spawn("bun", ["run", "watch:js"], { stdio: "inherit", shell: true }),
];

for (const child of children) {
  child.on("exit", (code) => {
    for (const other of children) {
      if (other !== child && !other.killed) {
        other.kill();
      }
    }
    process.exit(code ?? 1);
  });
}

process.on("SIGINT", () => {
  for (const child of children) {
    child.kill();
  }
  process.exit(0);
});
