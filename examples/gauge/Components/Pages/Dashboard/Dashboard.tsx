import { ComponentProps, DisplayComponent, FSComponent, Subscribable, VNode } from "@microsoft/msfs-sdk"
import { PackageInfo } from "@navigraph/msfs-navigation-data-interface"

interface DashboardProps extends ComponentProps {
  databases: Subscribable<PackageInfo[]>
}

export class Dashboard extends DisplayComponent<DashboardProps> {
  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-4xl">Dashboard</p>
        <div class="flex flex-row flex-grow flex-auto space-x-4">
          <div class="w-1/3 flex flex-col">
            <p class="text-2xl mb-4">Databases</p>
            <div class="mt-2 flex-grow rounded-xl bg-ng-background-500">
              {this.props.databases.map(val =>
                val.map(pkgs => (
                  <div class="p-2">
                    <p class="text-lg">
                      {pkgs.cycle.cycle} - {pkgs.cycle.format}
                    </p>
                  </div>
                )),
              )}
            </div>
          </div>
          <div class="flex flex-col flex-grow">
            <p class="text-2xl mb-4">Info</p>
            <div class="mt-2 flex-grow rounded-xl bg-ng-background-500"></div>
          </div>
        </div>
      </div>
    )
  }
}
