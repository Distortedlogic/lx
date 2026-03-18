import * as vscode from "vscode";
import { lxBinary } from "./diagram";

class LxTaskProvider implements vscode.TaskProvider {
  provideTasks(): Thenable<vscode.Task[]> {
    return this.buildTasks();
  }

  resolveTask(): vscode.Task | undefined {
    return undefined;
  }

  private async buildTasks(): Promise<vscode.Task[]> {
    const bin = lxBinary();
    const tasks: vscode.Task[] = [];

    tasks.push(this.makeTask("lx: run file", `${bin} run`, vscode.TaskGroup.Build));
    tasks.push(this.makeTask("lx: check workspace", `${bin} check`, vscode.TaskGroup.Build));
    tasks.push(this.makeTask("lx: test", `${bin} test`, vscode.TaskGroup.Test));
    tasks.push(this.makeTask("lx: list", `${bin} list`, vscode.TaskGroup.Build));

    const justfiles = await vscode.workspace.findFiles("justfile", null, 1);
    if (justfiles.length > 0) {
      tasks.push(this.makeTask("lx: just test", "just test", vscode.TaskGroup.Test));
      tasks.push(this.makeTask("lx: just diagnose", "just diagnose", vscode.TaskGroup.Build));
      tasks.push(this.makeTask("lx: just fmt", "just fmt", vscode.TaskGroup.Build));
      tasks.push(this.makeTask("lx: just build", "just build", vscode.TaskGroup.Build));
    }

    return tasks;
  }

  private makeTask(
    label: string,
    command: string,
    group: vscode.TaskGroup,
  ): vscode.Task {
    const def: vscode.TaskDefinition = { type: "lx" };
    const exec = new vscode.ShellExecution(command);
    const task = new vscode.Task(def, vscode.TaskScope.Workspace, label, "lx", exec);
    task.group = group;
    task.presentationOptions = { reveal: vscode.TaskRevealKind.Always };
    return task;
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.tasks.registerTaskProvider("lx", new LxTaskProvider()),
  );
}
