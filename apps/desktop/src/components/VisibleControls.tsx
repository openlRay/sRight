import { Checkbox, Switch } from '@heroui/react';
import { type ComponentProps, type ReactNode } from 'react';

type VisibleCheckboxProps = Omit<ComponentProps<typeof Checkbox>, 'children'> & {
    children?: ReactNode;
};
type VisibleSwitchProps = Omit<ComponentProps<typeof Switch>, 'children'> & {
    children?: ReactNode;
};

export function VisibleCheckbox({ children, variant = 'secondary', ...props }: VisibleCheckboxProps) {
    return (
        <Checkbox variant={variant} {...props}>
            <Checkbox.Control>
                <Checkbox.Indicator />
            </Checkbox.Control>
            {children ? <Checkbox.Content>{children}</Checkbox.Content> : null}
        </Checkbox>
    );
}

export function VisibleSwitch({ children, size = 'lg', ...props }: VisibleSwitchProps) {
    return (
        <Switch size={size} {...props}>
            <Switch.Control>
                <Switch.Thumb />
            </Switch.Control>
            {children ? <Switch.Content>{children}</Switch.Content> : null}
        </Switch>
    );
}
