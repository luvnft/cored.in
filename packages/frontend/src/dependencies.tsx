import {
  AuthService,
  FeedService,
  HttpService,
  IssuerService,
  StorageService
} from "@/services";
import { UserService } from "./modules/user/services";

interface customWindow extends Window {
  __API_URL__?: string;
}

declare const window: customWindow;
declare const __API_URL__: string;

const apiUrl = window.__API_URL__ || __API_URL__ || "/api/";

export const persistentStorageService = new StorageService();
const httpService = new HttpService(apiUrl);
export const authService = new AuthService(apiUrl);
export const userService = new UserService(httpService);
export const feedService = new FeedService(httpService);
export const issuerService = new IssuerService(httpService);
