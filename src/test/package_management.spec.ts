import { NavigraphNavigationDataInterface, PackageInfo } from "../js"

const navigationDataInterface = new NavigraphNavigationDataInterface()

describe("Package Management", () => {
    // This will run once for each test file
    beforeAll(async () => {
        const waitForReady = (navDataInterface: NavigraphNavigationDataInterface): Promise<void> => {
            return new Promise((resolve, _reject) => {
                navDataInterface.onReady(() => resolve())
            })
        }

        await waitForReady(navigationDataInterface)
    }, 30000)

    it("List packages contains bundled items", async () => {
        const packages = await navigationDataInterface.list_available_packages();

        const bundledV1 = packages.find((item) => item.cycle.cycle === '2101' && item.cycle.format === 'dfd');
        const bundledV2 = packages.find((item) => item.cycle.cycle === '2401' && item.cycle.format === 'dfdv2');

        expect(bundledV1).toStrictEqual({
            cycle: {
                cycle: '2101',
                format: 'dfd',
                name: "Navigraph Avionics",
                revision: "1",
                validityPeriod: "2021-01-25/2021-02-20",
            },
            is_bundled: true,
            path: "\\work/navigation-data/269b26b0-ba1b-3859-a9c0-4484dc766233",
            uuid: "269b26b0-ba1b-3859-a9c0-4484dc766233"
        } satisfies PackageInfo)

        expect(bundledV2).toStrictEqual({
            cycle: {
                cycle: '2401',
                format: 'dfdv2',
                name: "Navigraph Avionics",
                revision: "1",
                validityPeriod: "2024-01-25/2024-02-21",
            },
            is_bundled: true,
            path: "\\work/navigation-data/b8a9ecaa-a137-3059-b9f1-b7f2d5ddac37",
            uuid: "b8a9ecaa-a137-3059-b9f1-b7f2d5ddac37"
        } satisfies PackageInfo)
    })

    it("Clean up Packages", async () => {
        const active = await navigationDataInterface.get_active_package();

        await navigationDataInterface.clean_packages();

        let packages = await navigationDataInterface.list_available_packages();

        for (const item of packages) {
            expect(item.is_bundled == true || item.cycle.format == active?.cycle.format).toBe(true);
        }

        await navigationDataInterface.clean_packages(0);

        packages = await navigationDataInterface.list_available_packages();

        for (const item of packages) {
            expect(item.is_bundled == true || item.uuid == active?.uuid).toBe(true);
        }
    }, 40000);

    it("Delete packages", async () => {
        const intialPackages = await navigationDataInterface.list_available_packages();
        const activePackage = await navigationDataInterface.get_active_package();

        for (const item of intialPackages) {
            await navigationDataInterface.delete_package(item.uuid);

            const newPackages = await navigationDataInterface.list_available_packages();

            for (const newPackage of newPackages) {
                expect(newPackage.uuid !== item.uuid || item.uuid === activePackage?.uuid).toBe(true)
            }
        }

        expect(await navigationDataInterface.list_available_packages()).toHaveLength(activePackage ? 1 : 0);
    }, 30000);
})