<script setup lang="ts">
import { ref, onMounted, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { register } from "@tauri-apps/plugin-global-shortcut";
import { initStore, get, set } from "@/composables/StoreApp";
import VueSlider from "vue-3-slider-component";

onMounted(async () => {
  await initStore();
});

const msg = ref("");

const form = reactive<Record<string, number>>({
  zoom: 0,
  pan: 0,
  tilt: 0,
});

const cameras = ref([]);
const currentCamera = ref(0);

const profiles = ref<Array<number>>([1, 2, 3, 4, 5]);
const currentProfile = ref(0);

const methods = ref<Record<string, Record<string, number>>>({});

async function listCameras() {
  cameras.value = await invoke("list_cameras");
  let index = await get("index");
  currentCamera.value = index!.value;
}

async function saveCamera() {
  await set("index", { value: currentCamera.value });
  await invoke("set_camera", { device: currentCamera.value });
  msg.value = "Selected camera saved";

  setTimeout(() => (msg.value = ""), 10000);
}

async function callController(control: string) {
  msg.value = await invoke("control", {
    method: control,
    value: form[control],
  });

  setTimeout(() => (msg.value = ""), 2000);
}

async function controls() {
  await register("NUMADD", (event) => {
    if (event.state === "Pressed") {
      form["zoom"] += 1;
      callController("zoom");
    }
  });

  await register("NUMSUBTRACT", (event) => {
    if (event.state === "Pressed") {
      form["zoom"] -= 1;
      callController("zoom");
    }
  });

  await register(["CommandOrControl+Alt+LEFT", "NUM4"], (event) => {
    if (event.state === "Pressed") {
      form["pan"] += 1;
      callController("pan");
    }
  });

  await register(["CommandOrControl+Alt+RIGHT", "NUM6"], (event) => {
    if (event.state === "Pressed") {
      form["pan"] -= 1;
      callController("pan");
    }
  });

  await register(["CommandOrControl+Alt+UP", "NUM8"], (event) => {
    if (event.state === "Pressed") {
      form["tilt"] += 1;
      callController("tilt");
    }
  });

  await register(["CommandOrControl+Alt+DOWN", "NUM2"], (event) => {
    if (event.state === "Pressed") {
      form["tilt"] -= 1;
      callController("tilt");
    }
  });

  // reset position - PROFILE 0
  await register("CommandOrControl+DIGIT0", (event) => {
    if (event.state === "Pressed") {
      reset();
    }
  });

  // set each PROFILE shortcut
  await profiles.value.forEach(async (profile: number) => {
    await register("CommandOrControl+DIGIT" + profile, (event) => {
      if (event.state === "Pressed") {
        getProfile(profile);
      }
    });
  });
}

// -- Profiles handler --
// get profile from store
async function getProfile(profile: number) {
  let res: Record<string, number> = await get("profile_" + profile);
  currentProfile.value = profile;

  if (res) {
    for (let [k, v] of Object.entries(res)) {
      form[k] = v;

      callController(k);
    }
  }
}

// save profile to store
async function saveProfile() {
  await set("profile_" + currentProfile.value, form);
}

async function reset() {
  methods.value = await invoke("default");

  for (let [k, v] of Object.entries(methods.value)) {
    form[k] = v.val;

    callController(k);
  }
}

reset();
controls();
</script>

<template>
  <main class="container">
    <h1>Camera Control APP</h1>

    <div class="row">
      <img
        src="./assets/tauri.svg"
        class="logo app"
        @click.exact="listCameras"
      />
    </div>
    <div class="fade" v-show="cameras.length > 0">
      <p><b>Choose camera to control (USB / Bluetooth / WiFi) ....</b></p>

      <div class="row" style="padding: 10px">
        <select v-model="currentCamera">
          <option disabled value="">Choose camera ...</option>
          <option v-for="(k, camera) in cameras" :key="k" :value="camera">
            {{ k }}
          </option>
        </select>
        &nbsp;
        <button
          type="button"
          :disabled="currentCamera == null"
          @click.exact="saveCamera"
        >
          Save
        </button>
      </div>

      <p>
        <i>{{ msg }}</i>
      </p>

      <div style="margin: 10px">&nbsp;</div>

      <h3>Profiles</h3>

      <div class="row">
        <div v-for="profile in profiles" style="padding: 5px">
          <button
            type="button"
            style="padding: 25px"
            :class="currentProfile == profile ? 'active' : ''"
            @click.exact="getProfile(profile)"
          >
            {{ profile }}
          </button>
        </div>
      </div>
      <p></p>
      <div style="display: grid; justify-content: center">
        <div v-for="(opt, m) in methods">
          {{ m }}
          <VueSlider
            v-model="form[m]"
            width="500px"
            :min="opt.min"
            :max="opt.max"
            :dotSize="20"
            @drag-end="callController(m)"
            lazy
          />
        </div>
      </div>
      <p></p>
      <div class="row">
        <div>
          <button
            type="button"
            style="padding: 10px; background-color: red"
            @click.exact="reset"
          >
            Reset
          </button>
          &nbsp;
          <button
            type="button"
            style="padding: 10px"
            @click.exact="saveProfile"
          >
            Save profile
          </button>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.fade {
  opacity: 1;
  animation-name: fadeInOpacity;
  animation-iteration-count: 1;
  animation-timing-function: ease-in;
  animation-duration: 1s;
}

@keyframes fadeInOpacity {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

.logo.app:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.app:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
select,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active,
.active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
select,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  select,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active,
  .active {
    background-color: #0f0f0f69;
  }
}
</style>
