import type { Href, Router } from "expo-router";

export function exitToPreviousOrHome(router: Router, homePath: Href = "/") {
  if (router.canGoBack()) {
    router.back();
    return;
  }
  router.replace(homePath);
}
