import { ShellController, type ShellBridge } from "./state.js";

declare global { interface Window { __DSH_SHELL__?: ShellBridge } }
// This bridge is supplied by the native document-start shell integration.
// Its existence is not authorization: T024 validates actual WK sender metadata.
const unavailable = async (): Promise<never> => { throw new Error("Native shell unavailable."); };
const controller = new ShellController(window.__DSH_SHELL__ ?? { status: unavailable, acknowledge: unavailable, retry: unavailable, enter: unavailable });
function element<T extends HTMLElement>(id: string, kind: new (...args: never[]) => T): T {
  const node = document.getElementById(id);
  if (!(node instanceof kind)) throw new Error("Invalid shell document.");
  return node;
}
const enter = element("enter", HTMLButtonElement);
const retry = element("retry", HTMLButtonElement);
const openNotice = element("open-notice", HTMLButtonElement);
const accept = element("accept-notice", HTMLButtonElement);
const dismiss = element("dismiss-notice", HTMLButtonElement);
const dialog = element("notice", HTMLDialogElement);
const noticeTitle = element("notice-title", HTMLElement);
const noticeStatus = element("notice-status", HTMLElement);
const status = element("status", HTMLElement);
const nextAction = element("next-action", HTMLElement);
let returnFocus: HTMLElement = openNotice;
let prompted = false;
function render() {
  const view = controller.view;
  document.body.dataset.phase = view.phase;
  if (status.textContent !== view.message) status.textContent = view.message;
  nextAction.textContent = view.nextAction;
  enter.disabled = !view.canEnter;
  retry.hidden = !view.canRetry;
  accept.disabled = !view.canAcknowledge;
  dismiss.disabled = view.busy;
}
function showNotice() {
  returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : openNotice;
  noticeStatus.textContent = "";
  if (!dialog.open) { dialog.showModal(); noticeTitle.focus(); }
}
openNotice.addEventListener("click", showNotice);
dismiss.addEventListener("click", () => dialog.close());
dialog.addEventListener("cancel", (event) => { if (controller.view.busy) event.preventDefault(); });
dialog.addEventListener("close", () => { (returnFocus.isConnected && !(returnFocus instanceof HTMLButtonElement && returnFocus.disabled) ? returnFocus : openNotice).focus(); });
async function operate(action: () => Promise<void>) {
  const pending = action();
  render();
  await pending;
  render();
}
accept.addEventListener("click", async () => {
  await operate(() => controller.acknowledge());
  if (!controller.view.needsNotice) { returnFocus = enter.disabled ? openNotice : enter; dialog.close(); }
  else noticeStatus.textContent = controller.view.message;
});
retry.addEventListener("click", () => { void operate(() => controller.retry()); });
enter.addEventListener("click", () => { void operate(() => controller.enter()); });
async function refresh() {
  await controller.refresh();
  render();
  if (!prompted && controller.view.needsNotice) { prompted = true; showNotice(); }
  window.setTimeout(() => { void refresh(); }, 1000);
}
void refresh();
