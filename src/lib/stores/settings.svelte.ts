export interface Settings {
    export_format: string;
    base_export_dir: string;
    folder_format: string;
    custom_prefix: string;
    sub_folder: string;
    ask_every_time: boolean;
    current_theme: string;
}

export const defaultSettings: Settings = {
    export_format: 'wav',
    base_export_dir: '',
    folder_format: 'DD-MM-YYYY',
    custom_prefix: 'AudioLoom',
    sub_folder: 'Mixler',
    ask_every_time: true,
    current_theme: 'theme-modern',
};
export const playerState = $state<Settings>(defaultSettings);