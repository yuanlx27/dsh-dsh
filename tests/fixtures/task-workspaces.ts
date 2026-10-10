import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Disposable resources only; upstream R owns policies, tasks and execution. */
export async function taskWorkspaces() {
  const root = await mkdtemp(join(tmpdir(), "dsh-task-fixture-"));
  async function pair(owner: string) {
    const paths = { A: join(root, owner, "A"), B: join(root, owner, "B") };
    for (const path of Object.values(paths)) {
      await mkdir(path, { recursive: true, mode: 0o700 });
      await writeFile(join(path, "README.md"), `# Disposable ${owner} workspace\n\nOnly approval-marker.txt, actions.log and stop-marker.txt may be changed.\nNo credentials or production data belong here.\n`, { mode: 0o600 });
    }
    return paths;
  }
  try {
    return { root, standalone: await pair("standalone"), product: await pair("product"), close: () => rm(root, { recursive: true, force: true }) };
  } catch (error) { await rm(root, { recursive: true, force: true }); throw error; }
}
