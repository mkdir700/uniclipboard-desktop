import { createApi, fakeBaseQuery } from '@reduxjs/toolkit/query/react'
import { getOnboardingState, type OnboardingStatus } from '@/api/onboarding'
import { getEncryptionSessionStatus, type EncryptionSessionStatus } from '@/api/security'

type ApiError = {
  message: string
}

export const appApi = createApi({
  reducerPath: 'appApi',
  baseQuery: fakeBaseQuery<ApiError>(),
  tagTypes: ['EncryptionStatus', 'OnboardingStatus'],
  endpoints: builder => ({
    getEncryptionSessionStatus: builder.query<EncryptionSessionStatus, void>({
      queryFn: async () => {
        try {
          const data = await getEncryptionSessionStatus()
          return { data }
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error)
          return { error: { message } }
        }
      },
      providesTags: ['EncryptionStatus'],
    }),
    getOnboardingStatus: builder.query<OnboardingStatus, void>({
      queryFn: async () => {
        try {
          const data = await getOnboardingState()
          return { data }
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error)
          return { error: { message } }
        }
      },
      providesTags: ['OnboardingStatus'],
    }),
  }),
})

export const { useGetEncryptionSessionStatusQuery, useGetOnboardingStatusQuery } = appApi
