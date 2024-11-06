import { ComponentProps, DisplayComponent, FSComponent, Subscribable, VNode } from "@microsoft/msfs-sdk"

interface DashboardProps extends ComponentProps {}

export class Dashboard extends DisplayComponent<DashboardProps> {
  render(): VNode {
    return <p class="text-3xl">Dashboard</p>
  }
}
