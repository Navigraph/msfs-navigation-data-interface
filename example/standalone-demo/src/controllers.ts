import { Controller, Get, Param, ParseFloatPipe, Query } from "@nestjs/common";
import { ApiOperation, ApiParam, ApiQuery, ApiTags } from "@nestjs/swagger";
import { NavigationDataService } from "./NavigationDataService";

/** Documents the `:ident` path parameter */
const Ident = (example: string, description = "Identifier (case insensitive)") =>
  ApiParam({ name: "ident", example, description });

const AirportIdent = Ident("MMUN", "ICAO identifier of the airport (3-4 letters or digits, case insensitive)");

@ApiTags("Health")
@Controller("health")
export class HealthController {
  @Get()
  health() {
    return { status: "ok" };
  }
}

@ApiTags("Database")
@Controller("database")
export class DatabaseController {
  constructor(private readonly service: NavigationDataService) {}

  @Get()
  @ApiOperation({ summary: "Cycle and install status of the navigation data" })
  getDatabase() {
    return this.service.getDatabase();
  }
}

@ApiTags("Airports")
@Controller("airports")
export class AirportsController {
  constructor(private readonly service: NavigationDataService) {}

  @Get()
  @ApiOperation({ summary: "Airports within a range (NM) of a point" })
  @ApiQuery({ name: "lat", example: 21.04 })
  @ApiQuery({ name: "long", example: -86.87 })
  @ApiQuery({ name: "range", example: 50, description: "Nautical miles, at most 500" })
  getAirportsInRange(
    @Query("lat", ParseFloatPipe) lat: number,
    @Query("long", ParseFloatPipe) long: number,
    @Query("range", ParseFloatPipe) range: number,
  ) {
    return this.service.getAirportsInRange(lat, long, range);
  }

  @Get(":ident")
  @ApiOperation({ summary: "An airport" })
  @AirportIdent
  getAirport(@Param("ident") ident: string) {
    return this.service.getAirport(ident);
  }

  @Get(":ident/runways")
  @ApiOperation({ summary: "Runways of an airport" })
  @AirportIdent
  getRunways(@Param("ident") ident: string) {
    return this.service.getRunways(ident);
  }

  @Get(":ident/departures")
  @ApiOperation({ summary: "Departures (SIDs) of an airport" })
  @AirportIdent
  getDepartures(@Param("ident") ident: string) {
    return this.service.getDepartures(ident);
  }

  @Get(":ident/arrivals")
  @ApiOperation({ summary: "Arrivals (STARs) of an airport" })
  @AirportIdent
  getArrivals(@Param("ident") ident: string) {
    return this.service.getArrivals(ident);
  }

  @Get(":ident/approaches")
  @ApiOperation({ summary: "Approaches of an airport" })
  @AirportIdent
  getApproaches(@Param("ident") ident: string) {
    return this.service.getApproaches(ident);
  }

  @Get(":ident/gates")
  @ApiOperation({ summary: "Gates of an airport" })
  @AirportIdent
  getGates(@Param("ident") ident: string) {
    return this.service.getGates(ident);
  }

  @Get(":ident/communications")
  @ApiOperation({ summary: "Communication frequencies of an airport" })
  @AirportIdent
  getCommunications(@Param("ident") ident: string) {
    return this.service.getCommunications(ident);
  }
}

/** Enroute lookups return every match, as identifiers are not unique */
@ApiTags("Enroute")
@Controller()
export class EnrouteController {
  constructor(private readonly service: NavigationDataService) {}

  @Get("waypoints/:ident")
  @ApiOperation({ summary: "Waypoints with an identifier" })
  @Ident("COSTA")
  getWaypoints(@Param("ident") ident: string) {
    return this.service.getWaypoints(ident);
  }

  @Get("navaids/vhf/:ident")
  @ApiOperation({ summary: "VHF navaids with an identifier" })
  @Ident("CUN")
  getVhfNavaids(@Param("ident") ident: string) {
    return this.service.getVhfNavaids(ident);
  }

  @Get("navaids/ndb/:ident")
  @ApiOperation({ summary: "NDB navaids with an identifier" })
  @Ident("SGA")
  getNdbNavaids(@Param("ident") ident: string) {
    return this.service.getNdbNavaids(ident);
  }

  @Get("airways/:ident")
  @ApiOperation({ summary: "Airways with an identifier" })
  @Ident("A317")
  getAirways(@Param("ident") ident: string) {
    return this.service.getAirways(ident);
  }
}
