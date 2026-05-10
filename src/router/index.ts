import { createRouter, createWebHashHistory } from "vue-router";
import SmartAdvice from "../views/SmartAdvice.vue";
import DepartmentSearch from "../views/DepartmentSearch.vue";
import SettingsLayout from "../views/Settings/SettingsLayout.vue";
import ConnectionSettings from "../views/Settings/ConnectionSettings.vue";
import PttypeSettings from "../views/Settings/PttypeSettings.vue";
import IcodeSettings from "../views/Settings/IcodeSettings.vue";
import ExportImportSettings from "../views/Settings/ExportImportSettings.vue";
import DepartmentSettings from "../views/Settings/DepartmentSettings.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: SmartAdvice },
    { path: "/department-search", component: DepartmentSearch },
    {
      path: "/settings",
      component: SettingsLayout,
      children: [
        { path: "", redirect: "/settings/connection" },
        { path: "connection", component: ConnectionSettings },
        { path: "pttype", component: PttypeSettings },
        { path: "icode", component: IcodeSettings },
        { path: "export-import", component: ExportImportSettings },
        { path: "department", component: DepartmentSettings },
      ],
    },
  ],
});

export default router;
