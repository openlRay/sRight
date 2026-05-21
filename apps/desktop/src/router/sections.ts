import {
    FilePlus2,
    Heart,
    Send,
    Settings,
    Wrench
} from "lucide-react";
import { type LucideIcon } from "lucide-react";

export const preferenceSections = [
    { id: "general", label: "通用设置", path: "/general", icon: Settings, iconClass: "theme-general" },
    { id: "new-file", label: "新建文件", path: "/new-file", icon: FilePlus2, iconClass: "theme-new-file" },
    { id: "send-to", label: "发送文件到...", path: "/send-to", icon: Send, iconClass: "theme-send-to" },
    { id: "favorites", label: "常用目录", path: "/favorites", icon: Heart, iconClass: "theme-favorites" },
    { id: "toolbox", label: "工具箱", path: "/toolbox", icon: Wrench, iconClass: "theme-toolbox" }
] satisfies Array<{
    id: string;
    label: string;
    path: string;
    icon: LucideIcon;
    iconClass: string;
}>;
