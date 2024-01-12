import { NavigraphNavdataInterface } from "../js"

const navdataInterface = new NavigraphNavdataInterface()

describe("test", () => {
  it("Database info", async () => {
    const info = await navdataInterface.get_database_info("KJFK")

    expect(info).toStrictEqual({
      airac_cycle: "2313",
      effective_from_to: ["28-12-2023", "25-01-2024"],
      previous_from_to: ["30-11-2023", "28-12-2023"],
    })
  })

  it("Fetch airport", async () => {
    const airport = await navdataInterface.get_airport("KJFK")

    expect(airport.ident).toBe("KJFK")
  })

  it("Get airports in range", async () => {
    const airports = await navdataInterface.get_airports_in_range({ lat: 51.468, long: -0.4551 }, 640)

    expect(airports.length).toBe(1686)
  })

  it("Get airways", async () => {
    const airways = await navdataInterface.get_airways("A1")

    expect(airways.length).toBe(5)
  })

  it("Get airways in range", async () => {
    const airways = await navdataInterface.get_airways_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(airways.length).toBe(27)
  })

  it("Get departures", async () => {
    const departures = await navdataInterface.get_departures_at_airport("NZCH")

    expect(departures.length).toBe(15)
  })

  it("Get Arrivals", async () => {
    const arrivals = await navdataInterface.get_arrivals_at_airport("NZCH")

    expect(arrivals.length).toBe(45)
  })
})
