import { EventBus, Subject } from "@microsoft/msfs-sdk";
import { User } from "navigraph/auth";
import { auth } from "../Lib/navigraph";

const SERVICE_ID = Math.random().toString(36).slice(2);

interface AuthSyncPayload {
  user: User | null;
  source: string;
}

export class AuthService {
  static readonly user = Subject.create<User | null>(null);
  static readonly initialized = Subject.create(false);

  static init(bus: EventBus) {
    auth.onAuthStateChanged(user => {
      AuthService.user.set(user);

      if (AuthService.initialized.get()) {
        bus.pub("auth_state_changed", { user, source: SERVICE_ID } as AuthSyncPayload, true);
      }

      AuthService.initialized.set(true);
    });

    bus.on("auth_state_changed", (data: AuthSyncPayload) => {
      if (data.source !== SERVICE_ID) AuthService.user.set(data.user);
    });
  }

  static signIn = auth.signInWithDeviceFlow;
  static signOut = auth.signOut;
  static getUser = auth.getUser;
}
