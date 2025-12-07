# local-guide

## Team Information
| Name           | Student Number | Email                |
|----------------|---------------|----------------------|
| Yiduo Jing  | 1000308142      | yiduo.jing@mail.utoronto.ca    |
| Sitao Wang    | 1003695151      | sitao.wang@mail.utoronto.ca      |

See the video demo.mp4 located in the assets folder or go to the shared link [demo.mp4](link).

## Motivation
With the rise of the internet and social media, people increasingly enjoy taking photos at popular, internet-famous locations. However, most current apps prioritize broad, global recommendations over personalized local exploration. This highlights the need for a lightweight mobile tool that enables users to record, revisit, and organize their own local discoveries in a single, convenient platform. Our app addresses this need by offering a straightforward and user-friendly platform for organizing local points of interest. The target users are local adventurers who enjoy exploring their city, trying new cafes, parks, or attractions, and want an easy way to record and remember those discoveries.

## Objectives
The project aims to create a Local Guide app for local adventurers, allowing them to add, view, and organize their local points of interest in a safe, simple, and intuitive way.

## Technical Stack
The application utilized **React Native** with **Expo** as its development framework, implemented in **TypeScript**. State management is handled by the **Redux Toolkit**, and data is persisted locally using **React Native Async Storage** to retain state across app restarts. The backend service has been developed using **Axum** in **Rust**, providing a RESTful API.

## Features
1. **User Authentication:**
   - **Feature:** Secure user login and account management.
   - **Technical Approach:** The frontend sends an OAuth2 request to Google using PKCE and redirects the request, including the code verifier, to the backend. The backend then authenticates the request and returns a JWT. This JWT is stored in secure storage. An authentication gate is used to protect the user interface.

2. **Points of interest management**
   - **Feature:** Users can add/edit/delete and view points of interest with name, category, location, images and description (note).
   - **Technical Approach:** Used core components like View, Text, Button, TextInput, etc. to manage POI creation/update/deletion. (Please refer to the user guide for more details.)
  
3. **Screen Navigation:**
   - **Feature:** There are primarily five types of screens: the Google Sign in Screen, the Home Screen, the Add/Edit screen, the Details Screen and a user Settings Screen for logging out and enabling notification. When not logged in, the navigation flow is Login --> Home. After logging in, the flow changes to Home <--> Add/Edit <--> Home or Home <--> Details or Home <--> Settings.
   - **Technical Approach:** Used Expo Router for file-based routing. All screens are organized using a stack navigator, and the back button functionality is supported, allowing users to exit to the previous or home screen if no previous screen is available.

4. **State Management and Persistence:**
   - **Feature:** Persist states for Authentication and Points of Interest.
   - **Technical Approach:** Utilized the Redux Toolkit for global state management throughout the app, enabling shared access to POI data across multiple screens. User authentication and POI data are synchronized with the backend service to maintain consistency between the client and the backend. Points of interest are stored locally using React Native Async Storage to retain state, while authentication information is stored using SecureStorage.

5. **Notifications:**
   - **Feature:** Users can toggle to send a local notification daily at 9 a.m. after they enable the notification, to remind them to visit places and add points of interest. Tapping the notification takes users to the Home Screen.
   - **Technical Approach:** Utilized Expo Notifications and Permissions to set a trigger for a specific time, 9 a.m. every day, and the Switch component to toggle the notification.

6. **Backend Integration:**
   - **Feature:** A backend service has been developed using Axum in Rust, which provides a RESTful API. This allows users to Create, Read, Update, and Delete (CRUD) data related to places, including image handling via fetch requests.

   - **Technical Approach:** For detailed information regarding the API, including request headers, expected responses, and error handling for each endpoint, please refer to the complete API reference available in the documentation located at [backend/docs/backend/api.md](https://github.com/nichi1114/local-guide/blob/aa1d4ae3e9fdd827b13e6f2343c78243374fb356/docs/backend/api.md).

7. **Expo Location to show nearby places**
   - **Feature:** Users should see an enabled "Use Current Location" button that allows them to set their current location as the POI's location when adding or updating. This feature is available if they allow location access on their device.
   - **Technical Approach:** Used Expo Location to request users' permission and poll their current location (GPS coordinates). A human-readable address is converted using the Location.reverseGeocodeAsync function.

8. **Use Expo ImagePicker to select or capture photos of places:**
   - **Feature:** Users should see two buttons: one for selecting images from their library and another for taking a photo with the built-in camera. Both options require user permissions.
   - **Technical Approach:** Used Expo ImagePicker to request permissions and capture photos. The photos are stored in Async Storage and synchronized with the backend.

9. **Deployment:**  
   - **Feature:** Build and deploy the app.
   - **Technical Approach:** Set Up Expo EAS Build, configure backend keys and app metadata, and build the app for Android platform.

This project met the advanced features for **User Authentication** and **Mobile Sensors or Device APIs**. Core features, including POI management, navigation, state management, and backend integration, were completed on schedule, fulfilling the core technical requirement of utilizing React Native with Expo as the framework, developed in TypeScript, and employed Expo Router for navigation, Redux Toolkit for state management, and React Native Async Storage for data persistence.

## User Guide
All screens support a theme switch. Only images with a light theme will be displayed below.

### Google Signin: `/login`

To log in with your Google account, click the "Continue with Google" button. A successful login will redirect you to the home page.

<img src="./assets/demo_images/signin_page.jpg" width="300">


### Home: `/`

On this screen, you will find two buttons. The "Add" button will take you to the add place page, while the "Accounts" button will redirect you to the settings page.

A list of cards displaying places of interest is shown on this screen, featuring only the name, category, and location. Each card includes an "Edit" button, which redirects you to the edit place page, and a "Delete" button for removing the corresponding place.

<img src="./assets/demo_images/home_page.jpg" width="300">

### Settings: `/settings`

Inside your Account card, you can see your username and email address. You can toggle the daily local notification reminder using the switch button. The logout button allows you to log out of your session.

<img src="./assets/demo_images/settings_page.jpg" width="300">

### Add/Edit: `/add-edit`
The add/edit page lets you input the following information: name (required), category (required), location (required), images (optional), and notes (optional) for the point of interest (POI). You can enter your current location by clicking the "Use Current Location" button. Images can be selected from your library using the image button, or you can capture them with the camera button. You can delete certain images by clicking the trash-can button.

| add/edit page blank | add/edit page with image |
|-------|-------|
| <img src="./assets/demo_images/add_page.jpg" width="300"> | <img src="./assets/demo_images/add_page_image.jpg" width="300"> |

### Details: `places/[id]`

When you tap on a specific POI card, you will be redirected to the details screen. On the detail page, you can view all the details of the selected place. Additionally, you can click "Edit" to modify this place on the edit page, or click "Delete" to delete it and navigate back to the home screen.

<img src="./assets/demo_images/detail_page.jpg" width="300">

## Development Guide

### Deployed Version
Access the build at [ link ]. For more details about the build, refer to the **Deployment** section.

### Local Environment

Follow the steps below to stand up the API with a local Postgres instance and run the backend and app locally.

#### 1. Prepare Postgres and schema

Choose one of the two supported flows:

**Option A – Start your own Postgres container**
- Launch Postgres yourself. Example:

```sh
docker run --rm -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=local_guide \
  postgres:15
```

- Once the server is accepting connections, apply the schema by running `backend/sql/init.sql`:

```sh
psql -U postgres -d local_guide -f backend/sql/init.sql
```

**Option B – Helper script**
- Run the helper:

```sh
./scripts/backend/start-postgres.sh
```

The script boots a disposable Docker container named `local-guide-postgres` and automatically feeds it the same `init_db.sql` schema.

#### 2. Configure environment variables

Both the backend (Rust) service and the Expo frontend read configuration from a `.env` file at the repository root. Create that file manually (or via your preferred secrets manager) using the template below, then replace the placeholder values:

```env
BACKEND_URL=http://<local-ip-address>:8080
#BACKEND_BIND_ADDR=0.0.0.0:8080
DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide
#TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/local_guide_test
JWT_SECRET=replace-with-a-random-secret
JWT_TTL_SECONDS=3600

# Google OAuth – Android (required for Android builds)
GOOGLE_ANDROID_CLIENT_ID=<android-google-client-id>
GOOGLE_ANDROID_REDIRECT_URI=com.ece1778.localguide:/oauthredirect
#GOOGLE_ANDROID_PROVIDER_NAME=google-android

# Google OAuth – iOS (required for iOS builds)
GOOGLE_IOS_CLIENT_ID=<ios-google-client-id>
GOOGLE_IOS_REDIRECT_URI=com.ece1778.localguide:/oauthredirect
#GOOGLE_IOS_PROVIDER_NAME=google-ios

# Optional provider override
#GOOGLE_PROVIDER_NAME=google
#GOOGLE_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
#GOOGLE_TOKEN_URL=https://oauth2.googleapis.com/token
#GOOGLE_USERINFO_URL=https://www.googleapis.com/oauth2/v3/userinfo
```

The Expo client automatically hits `/auth/<provider>/callback`, where `<provider>` becomes `google-ios` or `google-android` (and `google` only if you also configure the shared web client), so be sure the backend has matching values for every platform you plan to support.

`BACKEND_URL` should point to the public base URL (what the frontend uses), while `BACKEND_BIND_ADDR` controls which host/port the Axum server listens on. Leave `BACKEND_BIND_ADDR` unset to keep the default `0.0.0.0:8080`. Environment variables exported directly in your shell still override `.env`, which can be handy for short-lived overrides or CI.

#### 3. Start the backend

From the repository root run:

```sh
cargo run --bin local-guide-backend
```

The server listens on `0.0.0.0:8080` by default.

#### 4. Start the app locally

From the repository root run:

```
npm expo start
```

Use **Expo Go** to open the app.


## Deployment Information
Downloadable URL: [ link ]
Details: [ .. ]


## Individual Contributions
Yiduo Jing completed the frontend POI management, state management, persistence, and the expo location and image picker features.

Sitao Wang developed the Rust backend service, which includes user authentication, user and places APIs, and the frontend settings screen, as well as local notifications and UI polish. He also handled the deployment.


## Lessons Learned and Concluding Remarks
This project has been valuable in enhancing our understanding of React Native development with Expo, especially in relation to Redux Toolkit and backend integration. However, we still find it somewhat challenging to work with `useSelector` particularly when we encounter warnings about selectors returning a new reference (such as an object or an array) that need to be memoized, especially when using the filter array method. Additionally, integrating with the backend has been an important lesson from this project. Overall, we enjoyed this course and the process of creating this project!