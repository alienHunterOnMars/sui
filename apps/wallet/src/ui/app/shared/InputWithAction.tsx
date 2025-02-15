// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { Text } from '_app/shared/text';
import NumberInput from '_components/number-input';
import { cva, type VariantProps } from 'class-variance-authority';
import { useField, useFormikContext } from 'formik';
import type { ComponentProps, ReactNode } from 'react';
import { forwardRef } from 'react';

import Alert from '../components/alert';
import { Pill, type PillProps } from './Pill';

const styles = cva(
	[
		'transition flex flex-row items-center p-3 bg-white text-body font-semibold',
		'placeholder:text-gray-60 w-full pr-[calc(20%_+_24px)] shadow-button',
		'border-solid border border-gray-45 text-steel-darker hover:border-steel focus:border-steel',
		'disabled:border-gray-40 disabled:text-gray-55',
	],
	{
		variants: {
			rounded: {
				lg: 'rounded-2lg',
				md: 'rounded-md',
			},
			// TODO: handle dark outline Pill
			dark: {
				true: '',
				false: '',
			},
		},
		defaultVariants: {
			rounded: 'lg',
			dark: false,
		},
	},
);

type ActionButtonProps = {
	actionText?: string;
	onActionClicked?: PillProps['onClick'];
	actionType?: PillProps['type'];
	name: string;
	actionDisabled?: boolean | 'auto';
};

export type InputWithActionProps = VariantProps<typeof styles> &
	(
		| (Omit<ComponentProps<'input'>, 'className' | 'type'> & {
				type?: 'text' | 'number' | 'password' | 'email';
		  })
		| (Omit<ComponentProps<typeof NumberInput>, 'form' | 'field' | 'meta'> & {
				type: 'numberInput';
		  })
	) &
	ActionButtonProps;

export function InputWithAction({
	actionText,
	onActionClicked,
	actionType = 'submit',
	type,
	disabled = false,
	actionDisabled = false,
	name,
	dark,
	rounded,
	...props
}: InputWithActionProps) {
	const [field, meta] = useField(name);
	const form = useFormikContext();
	const { isSubmitting } = form;
	const isInputDisabled = isSubmitting || disabled;
	const isActionDisabled =
		actionDisabled === 'auto'
			? isInputDisabled || meta?.initialValue === meta?.value || !!meta?.error
			: actionDisabled;

	return (
		<>
			<div className="flex flex-row flex-nowrap items-center relative">
				{type === 'numberInput' ? (
					<NumberInput
						className={styles({ rounded })}
						allowNegative
						{...props}
						form={form}
						field={field}
						meta={meta}
						disabled={isInputDisabled}
					/>
				) : (
					<input
						type={type}
						disabled={isInputDisabled}
						{...field}
						{...props}
						className={styles({ rounded })}
					/>
				)}
				<div className="flex items-center justify-end absolute right-0 max-w-[20%] mx-3 overflow-hidden">
					<Pill
						text={actionText}
						type={actionType}
						disabled={isActionDisabled}
						loading={isSubmitting}
						onClick={onActionClicked}
						dark={dark}
					/>
				</div>
			</div>

			{(meta?.touched && meta?.error) || (meta.value !== '' && meta.error) ? (
				<div className="mt-3">
					<Alert>{meta?.error}</Alert>
				</div>
			) : null}
		</>
	);
}

type InputWithActionZodFormProps = VariantProps<typeof styles> &
	(Omit<ComponentProps<'input'>, 'className' | 'type'> & {
		type?: 'text' | 'number' | 'password' | 'email';
	}) &
	ActionButtonProps & {
		errorString?: string;
		suffix?: ReactNode;
	};

export const InputWithActionButton = forwardRef<HTMLInputElement, InputWithActionZodFormProps>(
	(
		{
			actionText,
			onActionClicked,
			actionType = 'submit',
			type,
			disabled = false,
			actionDisabled = false,
			dark,
			rounded,
			errorString,
			value,
			suffix,
			...props
		},
		forwardRef,
	) => {
		return (
			<>
				<div className="flex flex-row flex-nowrap items-center relative">
					<input
						{...props}
						type={type}
						className={styles({ rounded })}
						disabled={disabled}
						ref={forwardRef}
					/>
					{suffix && value && (
						<div className="absolute z-0 flex h-full max-w-full items-center pl-3">
							<span className="invisible max-w-full text-sm">{value}</span>
							<span>
								<Text variant="body" color="steel">
									{suffix}
								</Text>
							</span>
						</div>
					)}

					<div className="flex items-center justify-end absolute right-0 max-w-[20%] mx-3 overflow-hidden">
						<Pill
							text={actionText}
							type={actionType}
							disabled={disabled}
							onClick={onActionClicked}
							dark={dark}
						/>
					</div>
				</div>

				{errorString ? (
					<div className="mt-3">
						<Alert>{errorString}</Alert>
					</div>
				) : null}
			</>
		);
	},
);
