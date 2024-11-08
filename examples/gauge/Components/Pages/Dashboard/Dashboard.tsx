import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  MappedSubject,
  MappedSubscribable,
  Subscribable,
  SubscribableArray,
  VNode,
} from "@microsoft/msfs-sdk"
import { NavigraphNavigationDataInterface, PackageInfo } from "@navigraph/msfs-navigation-data-interface"
import { List } from "../../List"
import { Button, InterfaceNavbarItemV2, InterfaceSwitch } from "../../Utils"

interface DashboardProps extends ComponentProps {
  databases: SubscribableArray<PackageInfo>
  selectedDatabase: Subscribable<PackageInfo | null>
  selectedDatabaseIndex: Subscribable<number>
  setSelectedDatabase: (database: PackageInfo) => void
  setSelectedDatabaseIndex: (index: number) => void
  activeDatabase: Subscribable<PackageInfo | null>
  interface: NavigraphNavigationDataInterface
}

export class Dashboard extends DisplayComponent<DashboardProps> {
  private readonly _selectedCallback = this.props.selectedDatabaseIndex.map(val => {
    if (this.props.databases.length !== 0) {
      this.props.setSelectedDatabase(this.props.databases.get(val))
    }
  })
  private readonly showActiveDatabase = MappedSubject.create(
    ([selectedDatabase, activeDatabase]) => selectedDatabase?.uuid === activeDatabase?.uuid,
    this.props.selectedDatabase,
    this.props.activeDatabase,
  )

  private displayItems(data: PackageInfo, index: number): VNode {
    return (
      <InterfaceNavbarItemV2
        class="w-full p-4 flex items-center"
        activeClass="bg-blue-400"
        content={""}
        active={this.props.selectedDatabaseIndex.map(val => val === index)}
        setActive={() => this.props.setSelectedDatabaseIndex(index)}
      >
        <p class="text-2xl text-inherit">
          {data.cycle.cycle} - {data.cycle.format}
        </p>
      </InterfaceNavbarItemV2>
    )
  }

  private setDatabase() {
    const selectedDatabase = this.props.selectedDatabase.get()

    if (selectedDatabase === null) {
      return
    }

    this.props.interface
      .set_active_package(selectedDatabase.uuid)
      .then(_res => {})
      .catch(err => console.error(err))
  }

  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="ml-2 mb-8 text-4xl">Dashboard</p>
        <div class="flex flex-row flex-grow flex-auto">
          <div class="w-1/3 flex flex-col">
            <p class="ml-2 text-3xl mb-4">Databases</p>
            <div class="mt-2 flex-grow bg-ng-background-500 shadow-inner">
              <div class="flex flex-col space-y-2">
                <List
                  data={this.props.databases}
                  renderItem={(data, index) => this.displayItems(data as PackageInfo, index)}
                />
              </div>
            </div>
            <Button onClick={() => this.setDatabase()} class="p-4 bg-blue-400">
              <p class="text-2xl">Select Database</p>
            </Button>
          </div>
          <ActiveDatabase selectedDatabase={this.props.selectedDatabase} showActiveDatabase={this.showActiveDatabase} />
        </div>
      </div>
    )
  }
}

interface ActiveDatabaseProps extends ComponentProps {
  selectedDatabase: Subscribable<PackageInfo | null>
  showActiveDatabase: MappedSubscribable<boolean>
}

class ActiveDatabase extends DisplayComponent<ActiveDatabaseProps> {
  private readonly isActive = this.props.showActiveDatabase.map(val => (val ? 0 : 1))

  render(): VNode {
    return (
      <div class="w-2/3 flex flex-col">
        <InterfaceSwitch
          class="ml-2 flex flex-row"
          active={this.isActive}
          noTheming={true}
          intoNoTheming={true}
          pages={[
            [0, <p class="text-3xl mb-4">Active Database</p>],
            [1, <p class="text-3xl mb-4">Selected Database</p>],
          ]}
        />
        <div class="mt-2 flex-grow bg-ng-background-700">
          <div class="p-4 flex flex-col align-middle items-start flex-start space-y-6 vertical">
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Bundled</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.is_bundled)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Installed format</p>
              <p class="text-2xl text-gray-400">
                {this.props.selectedDatabase.map(s => `${s?.cycle.format} revision ${s?.cycle.revision}`)}
              </p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active path</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.path)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Active cycle</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.cycle.cycle)}</p>
            </div>
            <div class="flex flex-col space-y-2">
              <p class="text-3xl">Validity period</p>
              <p class="text-2xl text-gray-400">{this.props.selectedDatabase.map(s => s?.cycle.validityPeriod)}</p>
            </div>
          </div>
        </div>
      </div>
    )
  }
}
