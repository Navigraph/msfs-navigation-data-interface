import { ComponentProps, DisplayComponent, FSComponent, Subscribable, VNode } from "@microsoft/msfs-sdk";
import { NavigationDataStatus, NavigraphNavigationDataInterface } from "@navigraph/msfs-navigation-data-interface";

interface DashboardProps extends ComponentProps {
  databaseInfo: Subscribable<NavigationDataStatus | null>;
  interface: NavigraphNavigationDataInterface;
}

export class Dashboard extends DisplayComponent<DashboardProps> {
  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="ml-2 mb-8 text-4xl">Dashboard</p>
        <div class="flex flex-row flex-grow flex-auto">
          <ActiveDatabase databaseInfo={this.props.databaseInfo} />
        </div>
      </div>
    );
  }
}

interface ActiveDatabaseProps extends ComponentProps {
  databaseInfo: Subscribable<NavigationDataStatus | null>;
  // : MappedSubscribable<boolean>
}

class ActiveDatabase extends DisplayComponent<ActiveDatabaseProps> {
  render(): VNode {
    return (
      <div class="w-2/3 flex flex-col">
        <p class="text-3xl mb-4">Active Database</p>
        <div class="mt-2 flex-grow bg-ng-background-700 overflow-auto">
          <div class="p-4 flex flex-col align-middle items-start flex-start space-y-6 vertical">
            <div class="flex flex-row space-x-2">
              <span class="text-2xl">Latest Cycle:</span>
              <span class="text-xl text-gray-400">{this.props.databaseInfo.map(s => s?.latestCycle ?? "N/A")}</span>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Bundled</p>
              <p class="text-2xl text-gray-400">{this.props.databaseInfo.map(s => s?.status ?? "N/A")}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Installed format</p>
              <p class="text-2xl text-gray-400">
                {this.props.databaseInfo.map(
                  s => `${s?.installedFormat ?? "N/A"} revision ${s?.installedRevision ?? "N/A"}`,
                )}
              </p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active path</p>
              <p class="text-2xl text-gray-400">{this.props.databaseInfo.map(s => s?.installedPath ?? "N/A")}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active cycle</p>
              <p class="text-2xl text-gray-400">{this.props.databaseInfo.map(s => s?.installedCycle ?? "N/A")}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Validity period</p>
              <p class="text-2xl text-gray-400">{this.props.databaseInfo.map(s => s?.validityPeriod ?? "N/A")}</p>
            </div>
          </div>
        </div>
      </div>
    );
  }
}
