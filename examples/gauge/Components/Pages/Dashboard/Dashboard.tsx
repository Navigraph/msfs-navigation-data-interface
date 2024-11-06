import { ComponentProps, DisplayComponent, FSComponent, MappedSubject, Subscribable, VNode } from "@microsoft/msfs-sdk"
import { PackageInfo } from "@navigraph/msfs-navigation-data-interface"

interface DashboardProps extends ComponentProps {
  databases: Subscribable<PackageInfo[]>
  activeDatabase: Subscribable<PackageInfo | null>
}

export class Dashboard extends DisplayComponent<DashboardProps> {
  public renderDatabaseInfo(): VNode | void {
    return (
      <>
        <div
          class={this.props.activeDatabase.map(
            status =>
              `p-4 flex flex-col align-middle items-start flex-start space-y-6 ${status ? "vertical" : "hidden"}`,
          )}
        >
          <div class="flex flex-col space-y-2">
            <p class="text-3xl">Bundled</p>
            <p class="text-2xl text-gray-400">{this.props.activeDatabase.map(s => s?.is_bundled)}</p>
          </div>
          <div class="flex flex-col space-y-2">
            <p class="text-3xl">Installed format</p>
            <p class="text-2xl text-gray-400">
              {this.props.activeDatabase.map(s => `${s?.cycle.format} revision ${s?.cycle.revision}`)}
            </p>
          </div>
          <div class="flex flex-col space-y-2">
            <p class="text-3xl">Active path</p>
            <p class="text-2xl text-gray-400">{this.props.activeDatabase.map(s => s?.path)}</p>
          </div>
          <div class="flex flex-col space-y-2">
            <p class="text-3xl">Active cycle</p>
            <p class="text-2xl text-gray-400">{this.props.activeDatabase.map(s => s?.cycle.cycle)}</p>
          </div>
          <div class="flex flex-col space-y-2">
            <p class="text-3xl">Validity period</p>
            <p class="text-2xl text-gray-400">{this.props.activeDatabase.map(s => s?.cycle.validityPeriod)}</p>
          </div>
        </div>
      </>
    )
  }

  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-6xl">Dashboard</p>
        <div class="flex flex-row flex-grow flex-auto space-x-4">
          <div class="w-1/3 flex flex-col">
            <p class="text-3xl mb-4">Databases</p>
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
            <p class="text-3xl mb-4">Info</p>
            <div class="mt-2 flex-grow rounded-xl bg-ng-background-500">{this.renderDatabaseInfo()}</div>
          </div>
        </div>
      </div>
    )
  }
}
