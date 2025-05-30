/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file was automatically generated by TanStack Router.
// You should NOT make any changes in this file as it will be overwritten.
// Additionally, you should also exclude this file from your linter and/or formatter to prevent it from being checked or modified.

// Import Routes

import { Route as rootRoute } from './routes/__root'
import { Route as SettingsImport } from './routes/settings'
import { Route as ScreenerImport } from './routes/screener'
import { Route as PortfolioImport } from './routes/portfolio'
import { Route as ChatImport } from './routes/chat'
import { Route as IndexImport } from './routes/index'

// Create/Update Routes

const SettingsRoute = SettingsImport.update({
  id: '/settings',
  path: '/settings',
  getParentRoute: () => rootRoute,
} as any)

const ScreenerRoute = ScreenerImport.update({
  id: '/screener',
  path: '/screener',
  getParentRoute: () => rootRoute,
} as any)

const PortfolioRoute = PortfolioImport.update({
  id: '/portfolio',
  path: '/portfolio',
  getParentRoute: () => rootRoute,
} as any)

const ChatRoute = ChatImport.update({
  id: '/chat',
  path: '/chat',
  getParentRoute: () => rootRoute,
} as any)

const IndexRoute = IndexImport.update({
  id: '/',
  path: '/',
  getParentRoute: () => rootRoute,
} as any)

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/': {
      id: '/'
      path: '/'
      fullPath: '/'
      preLoaderRoute: typeof IndexImport
      parentRoute: typeof rootRoute
    }
    '/chat': {
      id: '/chat'
      path: '/chat'
      fullPath: '/chat'
      preLoaderRoute: typeof ChatImport
      parentRoute: typeof rootRoute
    }
    '/portfolio': {
      id: '/portfolio'
      path: '/portfolio'
      fullPath: '/portfolio'
      preLoaderRoute: typeof PortfolioImport
      parentRoute: typeof rootRoute
    }
    '/screener': {
      id: '/screener'
      path: '/screener'
      fullPath: '/screener'
      preLoaderRoute: typeof ScreenerImport
      parentRoute: typeof rootRoute
    }
    '/settings': {
      id: '/settings'
      path: '/settings'
      fullPath: '/settings'
      preLoaderRoute: typeof SettingsImport
      parentRoute: typeof rootRoute
    }
  }
}

// Create and export the route tree

export interface FileRoutesByFullPath {
  '/': typeof IndexRoute
  '/chat': typeof ChatRoute
  '/portfolio': typeof PortfolioRoute
  '/screener': typeof ScreenerRoute
  '/settings': typeof SettingsRoute
}

export interface FileRoutesByTo {
  '/': typeof IndexRoute
  '/chat': typeof ChatRoute
  '/portfolio': typeof PortfolioRoute
  '/screener': typeof ScreenerRoute
  '/settings': typeof SettingsRoute
}

export interface FileRoutesById {
  __root__: typeof rootRoute
  '/': typeof IndexRoute
  '/chat': typeof ChatRoute
  '/portfolio': typeof PortfolioRoute
  '/screener': typeof ScreenerRoute
  '/settings': typeof SettingsRoute
}

export interface FileRouteTypes {
  fileRoutesByFullPath: FileRoutesByFullPath
  fullPaths: '/' | '/chat' | '/portfolio' | '/screener' | '/settings'
  fileRoutesByTo: FileRoutesByTo
  to: '/' | '/chat' | '/portfolio' | '/screener' | '/settings'
  id: '__root__' | '/' | '/chat' | '/portfolio' | '/screener' | '/settings'
  fileRoutesById: FileRoutesById
}

export interface RootRouteChildren {
  IndexRoute: typeof IndexRoute
  ChatRoute: typeof ChatRoute
  PortfolioRoute: typeof PortfolioRoute
  ScreenerRoute: typeof ScreenerRoute
  SettingsRoute: typeof SettingsRoute
}

const rootRouteChildren: RootRouteChildren = {
  IndexRoute: IndexRoute,
  ChatRoute: ChatRoute,
  PortfolioRoute: PortfolioRoute,
  ScreenerRoute: ScreenerRoute,
  SettingsRoute: SettingsRoute,
}

export const routeTree = rootRoute
  ._addFileChildren(rootRouteChildren)
  ._addFileTypes<FileRouteTypes>()

/* ROUTE_MANIFEST_START
{
  "routes": {
    "__root__": {
      "filePath": "__root.tsx",
      "children": [
        "/",
        "/chat",
        "/portfolio",
        "/screener",
        "/settings"
      ]
    },
    "/": {
      "filePath": "index.tsx"
    },
    "/chat": {
      "filePath": "chat.tsx"
    },
    "/portfolio": {
      "filePath": "portfolio.tsx"
    },
    "/screener": {
      "filePath": "screener.tsx"
    },
    "/settings": {
      "filePath": "settings.tsx"
    }
  }
}
ROUTE_MANIFEST_END */
