import { Navigate, createHashRouter } from 'react-router-dom';
import PreferenceShell from '../components/PreferenceShell';
import FavoritesView from '../views/FavoritesView';
import GeneralSettingsView from '../views/GeneralSettingsView';
import NewFileTemplatesView from '../views/NewFileTemplatesView';
import SendToView from '../views/SendToView';
import ToolboxView from '../views/ToolboxView';

const router = createHashRouter([
    {
        path: '/',
        element: <PreferenceShell />,
        children: [
            { index: true, element: <Navigate to="/general" replace /> },
            { path: 'general', element: <GeneralSettingsView /> },
            { path: 'new-file', element: <NewFileTemplatesView /> },
            { path: 'send-to', element: <SendToView /> },
            { path: 'favorites', element: <FavoritesView /> },
            { path: 'toolbox', element: <ToolboxView /> },
            { path: '*', element: <Navigate to="/general" replace /> }
        ]
    }
]);

export default router;
