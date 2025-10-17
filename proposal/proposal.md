# Local Guide

* Duration: 4.5 Weeks (development) + 0.5 Week (report + demo + buffer time)
* Team member: Yiduo Jing [1000308142], Sitao Wang [1003695151]

## Motivation
With the rise of the internet and social media, people increasingly enjoy taking photos at popular, internet-famous locations. However, most current apps prioritize broad, global recommendations over personalized local exploration. This highlights the need for a lightweight mobile tool that enables users to record, revisit, and organize their own local discoveries in a single, convenient platform. Our app addresses this need by offering a straightforward and user-friendly platform for organizing local points of interest. The target users are local adventurers who enjoy exploring their city, trying new cafes, parks, or attractions, and want an easy way to record and remember those discoveries.

## Objective and Key Features
The project aims to create a Local Guide app for local adventurers, allowing them to add, view, and organize their local points of interest in a safe, simple, and intuitive way.

The application will utilize **React Native** with **Expo** as its development framework, implemented in **TypeScript**. State management will be handled by the **Context API**, and data will be persisted locally using **React Native Async Storage** to retain state across app restarts.

**Core Features:**

1. **User Authentication:**
   - **Feature:** Secure user login and account management.
   - **Technical Approach:** Use Supabase Auth and support email/password login initially, with optional OAuth providers (e.g., Google) if time allows.

2. **Add and view points of interest**
   - **Feature:** Users can add and view points of interest with name, category, location, description (note), etc.
   - **Technical Approach:** Use core components like View, Text, Button, TextInput, etc. to manage POI creation/update. The category can be selected from a dropdown picker, preferably a tag-based picker, or fetched from Google Places if time permits. The location can be input as text or selected using the current location feature through Expo Location, or by utilizing a small map picker depending on the time.

3. **Screen Navigation:**
   - **Feature:** There are primarily four types of screens: the Signup/Login Screen, the Home Screen, the Add/Edit screen, and the Details Screen. When not logged in, the navigation flow is (Signup) <--> Login --> Home. After logging in, the flow changes to Home <--> Add/Edit <--> Home or Home <--> Details.
   - **Technical Approach:** Use Expo Router for file-based routing. All screens are organized using stack navigatior, and the back button functionality is supported. The folder structure will be organized as follows:
      ```plaintext
      app/
      ├── (auth)/
      │   ├── login.tsx
      │   ├── signup.tsx
      │   ├── _layout.tsx         → Auth layout
      ├── (main)/
      │   ├── index.tsx           → Home Screen
      │   ├── add-edit.tsx        → Add or update
      │   ├── place/[id].tsx      → Details Screen
      ├── _layout.tsx             → Root layout
      components/
      ├── ...
      ```
4. **State Management and Persistence:**
   - **Feature:** Persist states such as Authentication, Points of Interest, and UI elements (e.g., theme).
   - **Technical Approach:** Utilize the Context API for global state management throughout the app, enabling shared access to authentication and POI data across multiple screens. User authentication and data are synchronized with Supabase to maintain consistency between the client and the backend. Points of interest, along with lightweight UI states, are stored locally using React Native Async Storage to retain state.
   - **Screen Design Drafts**

      ![Signup Screen](https://github.com/nichi1114/local-guide/blob/main/proposal/signup_screen.png?raw=true)
      ![Login Screen](https://github.com/nichi1114/local-guide/blob/main/proposal/login_screen.png?raw=true)

      ![Home Screen](https://github.com/nichi1114/local-guide/blob/main/proposal/home_screen.png?raw=true)
      ![Add/Edit Screen](https://github.com/nichi1114/local-guide/blob/main/proposal/add_update_screen.png?raw=true)
      ![Details Screen](https://github.com/nichi1114/local-guide/blob/main/proposal/details_screen.png?raw=true)

5. **Notifications:**
   - **Feature:** Send a local notification at a scheduled time, either daily or weekly, to remind users to visit places and add points of interest. Tapping the notification takes users to the Home Screen.
   - **Technical Approach:** Utilize Expo Notifications and Permissions to set a trigger for a specific time, such as 9 a.m. every day.

6. **Backend Integration:**
   - **Feature:** Integrate the app with a backend service for persistant storage of POIs.
   - **Technical Approach:** Create a places table in Supabase with fields: id, user_id, name, category, description, latitude, longitude, and created_at. Enable Row-Level Security (RLS) so users can only access their own records. Use the Supabase client SDK in React Native for CRUD operations on points of interest. If time allows, consider using Supabase Realtime to subscribe to updates in the places table and refresh the UI instantly.

7. **Deployment:**  
   - **Feature:** Build and deploy the app.
   - **Technical Approach:** Set Up Expo EAS Build, configure Supabase keys and app metadata, and build the app for iOS or Android platform.

8. **Expo Location to show nearby places**
   - **Feature:** Users should see an enabled "Use Current Location" button that allows them to set their current location as the POI's location when adding or updating. This feature is available if they allow location access on their device. If time permits, a location filter tool could be implemented to show only places near the current location, and this tool could be added to the Home Screen.
   - **Technical Approach:** Use Expo Location to request users' permission and poll their current location (GPS coordinates).

9. **Use Expo Camera to capture photos of places (Optional):**
   - **Feature:** If time permits, we can support taking and uploading photos when adding or updating points of interest.
   - **Technical Approach:** Utilize Expo Camera to request permission and capture photos. Store the photos in Async Storage or Supabase storage.

This project meets the core requirements and advanced requirements for **User Authentication** and **Mobile Sensors or Device APIs**. It can be completed within a timeline of 4 to 5 weeks and will focus on essential functionalities such as navigation, view points of interest management, state management and persistence, and backend integration. Additional features, such as Expo Camera to capture photos, location filter tool, may be included as optional enhancements depending on the available time. What's more, UI design can be simplified if necessary to ensure the project is completed on time.

## Tentative Plan
The timeline below is generally planned, but it is highly possible to get adjusted.

**Week 1: Setup & User Authentication**

Yiduo:
   - Set up GitHub repo.
   - Initialize the Expo project and configure Expo Router with folder structure.
   - Integrate Supabase client.
   - Implement Signup & Login UI screens using Supabase Auth (email/password).
   - If time allows, implement OAuth provider.

Sitao:
   - Set up Supabase project: Create users and places tables.
   - Configure Row-Level Security (RLS) and policies.
   - Test basic CRUD via Supabase dashboard.
   - Prepare mock data for development.
   - Test navigation flow: (Signup) <--> Login --> Home.

**Week 2: Points of Interest Management**

Yiduo:
   - Create Add/Edit Screen UI with form fields (name, category, location, description).
   - Implement CRUD functions using Supabase client.
   - Use Async Storage to cache POIs and UI elements.

Sitao:
   - Build Home Screen to fetch and list user-specific POIs from Supabase.
   - Implement Details Screen for selected POI.
   - Set up Context API for shared state: authentication, POI data.
   - Test navigation: home <--> Add/Edit <--> Details.


**Week 3: Location & Notification**

Yiduo:
   - Integrate Expo Notifications: Request permission and schedule daily reminder (e.g., 9 a.m.).
   - Set up deep linking so tapping notification opens Home Screen.
   - Handle errors/loading states.

Sitao:
   - Implement Expo Location: Request location permission.
   - Add "Use Current Location" button on Add/Edit Screen.
   - Store latitude & longitude in the Supabase places table.
   - If time allows, begin implementing location-based filtering on Home Screen.


**Weeks 4-4.5: UI Polish, Testing & Optional Camera Feature**

Yiduo:
   - Integrate Expo Camera (optional feature): Request permission and capture image.
   - Store photo in Supabase Storage (or locally if short on time).
   - Add optional photo preview in Add/Edit and Details screen.
   - Test Supabase Storage uploads (if implemented).

Sitao:
   - Polish UI and navigation: responsive design and style consistency.
   - Perform testing across Android/iOS simulators.
   - Fix bugs and ensure state consistency.

**Weeks 4.5-5: Deployment, Demo & Report**

Yiduo:
   - Prepare EAS Build setup for Android/iOS.
   - Test deployed build on a real device.
   - Write the final report.

Sitao:
   - Polish the UI for an improved user experience.
   - Record the demo video showcasing all core features.
   - Write the final report.