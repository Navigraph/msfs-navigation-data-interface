import "reflect-metadata";
import {
  type ArgumentsHost,
  BadRequestException,
  Catch,
  type DynamicModule,
  type INestApplication,
  Module,
  type NestApplicationOptions,
  NotFoundException,
} from "@nestjs/common";
import { APP_FILTER, BaseExceptionFilter, NestFactory } from "@nestjs/core";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import { AirportsController, DatabaseController, EnrouteController, HealthController } from "./controllers";
import { NotFoundError, ValidationError } from "./errors";
import { type NavigationDataSource, NavigationDataService } from "./NavigationDataService";

/** Maps the service's errors to HTTP errors (400 and 404). Anything else is left to Nest, which responds 500. */
@Catch(ValidationError, NotFoundError)
class ServiceErrorFilter extends BaseExceptionFilter {
  override catch(error: ValidationError | NotFoundError, host: ArgumentsHost) {
    const exception =
      error instanceof ValidationError ? new BadRequestException(error.message) : new NotFoundException(error.message);
    super.catch(exception, host);
  }
}

@Module({})
export class AppModule {
  /**
   * @param source - The navigation data to serve: the JS interface, or a fake in tests
   */
  static register(source: NavigationDataSource): DynamicModule {
    return {
      module: AppModule,
      controllers: [HealthController, DatabaseController, AirportsController, EnrouteController],
      providers: [
        // The service is plain TS, created here rather than decorated, so it stays free of the framework
        { provide: NavigationDataService, useValue: new NavigationDataService(source) },
        { provide: APP_FILTER, useClass: ServiceErrorFilter },
      ],
    };
  }
}

/**
 * Creates the app, with the API under `/api`, Swagger UI at `/docs` and the OpenAPI spec at `/openapi.json`
 */
export async function createApp(
  source: NavigationDataSource,
  options?: NestApplicationOptions,
): Promise<INestApplication> {
  const app = await NestFactory.create(AppModule.register(source), options);
  app.setGlobalPrefix("api", { exclude: ["health"] });

  const config = new DocumentBuilder()
    .setTitle("Navigraph Navigation Data (standalone demo)")
    .setDescription(
      "Serves the mock navigation data of the standalone WASM module. Only the airports the mock data was generated for " +
        "(MMUN, MMMD and MSLP by default) and their surroundings are available.",
    )
    .setVersion("1.0.0")
    .build();
  SwaggerModule.setup("docs", app, () => SwaggerModule.createDocument(app, config), {
    jsonDocumentUrl: "openapi.json",
  });

  return app;
}
